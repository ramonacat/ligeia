use crate::{
    context::diagnostic::{DIAGNOSTIC_HANDLER, DiagnosticHandler},
    global_symbol::GlobalSymbol,
    types::RepresentedAs,
};
mod global_initializers;

use std::{
    collections::HashMap, error::Error, ffi::CString, fmt::Display, hash::Hash, rc::Rc,
    str::FromStr as _,
};

use llvm_sys::{
    LLVMLinkage,
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMDisposeModule, LLVMDumpModule, LLVMGetUndef,
        LLVMModuleCreateWithNameInContext, LLVMSetInitializer, LLVMSetLinkage,
    },
    prelude::{LLVMModuleRef, LLVMValueRef},
};
use thiserror::Error;

use super::{DeclaredFunctionDescriptor, ModuleId, built::Module};
use crate::{
    context::LLVM_CONTEXT,
    function::{
        builder::{FunctionBuilder, FunctionReference},
        declaration::{FunctionSignature, Visibility},
    },
    global_symbol::GlobalSymbols,
    module::builder::global_initializers::{GLOBAL_INITIALIZERS_ENTRY_TYPE, InitializersEntryType},
    package::context::PackageContext,
    types::{self, Type},
    value::{ConstValue, Value},
};

thread_local! {
    pub(super) static GLOBAL_INITIALIZER_TYPE: types::Function = types::Function::new(&<()>::representation(), &[]);
}

#[derive(Debug)]
pub struct ModuleBuildError {
    module_name: String,
    message: String,
    diagnostics: Vec<crate::context::diagnostic::Diagnostic>,
}

impl Display for ModuleBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to build the module \"{}\":\n{}\nDiagnosics:\n",
            self.module_name, self.message,
        )?;

        for diagnostic in &self.diagnostics {
            writeln!(f, "{diagnostic}")?;
        }

        Ok(())
    }
}

impl Error for ModuleBuildError {}

#[derive(Debug, Error)]
pub enum FunctionImportError {
    #[error("Function {0:?} is not exported")]
    NotExported(DeclaredFunctionDescriptor),
    #[error("Function {0:?} cannot be imported into the same module where it was defined")]
    DefinedInThisModule(DeclaredFunctionDescriptor),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct GlobalId(ModuleId, GlobalSymbol);

struct GlobalInitializerDescriptor {
    priority: u32,
    function: DeclaredFunctionDescriptor,
    initialized_data_pointer: Option<ConstValue>,
}

pub struct ModuleBuilder {
    id: ModuleId,
    reference: LLVMModuleRef,
    symbols: Rc<GlobalSymbols>,
    functions: HashMap<DeclaredFunctionDescriptor, LLVMValueRef>,
    global_initializers: Vec<GlobalInitializerDescriptor>,
    global_mappings: HashMap<String, usize>,
    globals: HashMap<GlobalId, LLVMValueRef>,
}

impl ModuleBuilder {
    pub(crate) fn new(package_context: &PackageContext, name: &str) -> Self {
        let module = LLVM_CONTEXT.with(|context| {
            let name = CString::from_str(name).unwrap();

            // SAFETY: The `name` is a valid null-terminated string, and we have a reference to
            // context, so the one returned from `as_llvm_ref` must be valid
            unsafe {
                LLVMModuleCreateWithNameInContext(name.as_ptr().cast(), context.as_llvm_ref())
            }
        });

        let symbols = package_context.symbols();

        Self {
            reference: module,
            id: ModuleId(package_context.id(), symbols.intern(name)),
            symbols,
            functions: HashMap::new(),
            global_initializers: vec![],
            global_mappings: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }

    // TODO: some cute macro so that these functions are easier to define?
    /// # Panics
    /// Will panic if the name cannot be converted into a c-string.
    /// # Safety
    /// The `runtime_function_address` must point at a function with `extern "C"` linkage, that
    /// matches the signature declared in `declaration`
    pub unsafe fn define_runtime_function(
        &mut self,
        declaration: &FunctionSignature,
        runtime_function_address: usize,
    ) -> DeclaredFunctionDescriptor {
        let id = DeclaredFunctionDescriptor {
            module_id: self.id,
            name: self.symbols.intern(declaration.name()),
            r#type: declaration.r#type(),
            visibility: declaration.visibility(),
        };

        let name = self.symbols.resolve(id.name);
        let c_name = CString::from_str(&name).unwrap();

        let function =
            // SAFETY: All the passed values come from objects which uphold guarantees about the
            // pointers being valid
            unsafe { LLVMAddFunction(self.reference, c_name.as_ptr(), id.r#type.as_llvm_ref()) };

        self.functions.insert(id, function);
        self.global_mappings.insert(name, runtime_function_address);

        id
    }

    // TODO: some cute macro so that these functions are easier to define?
    pub fn define_function(
        &mut self,
        declaration: &FunctionSignature,
        implement: impl FnOnce(&FunctionBuilder),
    ) -> DeclaredFunctionDescriptor {
        let id = DeclaredFunctionDescriptor {
            module_id: self.id,
            name: self.symbols.intern(declaration.name()),
            r#type: declaration.r#type(),
            visibility: declaration.visibility(),
        };
        let builder = FunctionBuilder::new(self, declaration);

        // TODO we should probably ask the builder to
        // verify that all blocks got built with at least a terminator
        implement(&builder);

        let function = builder.build();

        self.functions.insert(id, function);

        id
    }

    pub fn define_global_initializer(
        &mut self,
        name: &str,
        priority: u32,
        initialized_data_pointer: Option<ConstValue>,
        implement: impl FnOnce(&FunctionBuilder),
    ) {
        let function = GLOBAL_INITIALIZER_TYPE.with(|initializer| {
            self.define_function(
                &FunctionSignature::new(
                    format!("global_initializer_{name}"),
                    *initializer,
                    Visibility::Internal,
                ),
                implement,
            )
        });
        self.global_initializers.push(GlobalInitializerDescriptor {
            priority,
            function,
            initialized_data_pointer,
        });
    }

    /// # Panics
    /// Will panic if the name cannot be converted to a `CString`
    /// # Errors
    /// Will return an error if the function is defined in this module, or if the other module is
    /// not exporting it.
    pub fn import_function(
        &mut self,
        id: DeclaredFunctionDescriptor,
    ) -> Result<DeclaredFunctionDescriptor, FunctionImportError> {
        if id.module_id == self.id {
            return Err(FunctionImportError::DefinedInThisModule(id));
        }

        if id.visibility != Visibility::Export {
            return Err(FunctionImportError::NotExported(id));
        }

        let name = self.symbols.resolve(id.name);
        let c_name = CString::from_str(&name).unwrap();

        let function =
            // SAFETY: All the passed values come from objects which uphold guarantees about the
            // pointers being valid
            unsafe { LLVMAddFunction(self.reference, c_name.as_ptr(), id.r#type.as_llvm_ref()) };

        let id = DeclaredFunctionDescriptor {
            module_id: self.id,
            name: id.name,
            r#type: id.r#type,
            visibility: Visibility::Internal,
        };

        self.functions.insert(id, function);

        Ok(id)
    }

    pub(crate) fn build(mut self) -> Result<Module, ModuleBuildError> {
        self.build_global_initializers();
        // TODO expose an API so the user can do whatever they want with the IR dump
        // SAFETY: We have a valid, non-null `reference`, so this function can't fail
        unsafe { LLVMDumpModule(self.reference) };

        let mut out_message = std::ptr::null_mut();
        // SAFETY: We have a valid, non-null `reference`, and since the action is
        // `LLVMAbortProcessAction`, and `out_message` is passed as a pointer to a pointer, so
        // we'll get a new pointer put into there
        let verify_result = unsafe {
            LLVMVerifyModule(
                self.reference,
                LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &raw mut out_message,
            )
        };

        if verify_result != 0 {
            // SAFETY: We received the message from the verify call above, it must be a valid
            // pointer
            let message = unsafe { CString::from_raw(out_message) }
                .to_str()
                .unwrap()
                .to_string();

            let diagnostics = DIAGNOSTIC_HANDLER.with(DiagnosticHandler::take_diagnostics);
            return Err(ModuleBuildError {
                module_name: self.symbols.resolve(self.id.1),
                message,
                diagnostics,
            });
        }

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        let mut functions = HashMap::new();
        std::mem::swap(&mut functions, &mut self.functions);

        let mut global_mappings = HashMap::new();
        std::mem::swap(&mut global_mappings, &mut self.global_mappings);

        // SAFETY: We have ensured that the reference is not owned by this current object
        Ok(unsafe {
            Module::new(
                self.id,
                reference,
                functions,
                self.symbols.clone(),
                global_mappings,
            )
        })
    }

    pub(crate) fn get_function(
        &self,
        function: DeclaredFunctionDescriptor,
    ) -> FunctionReference<'_> {
        let value = self.functions.get(&function).unwrap();

        // SAFETY: The functions here were transfered from the ModuleBuilder, so we know they
        // belong to this module, so as long as the function reference has a life time at least
        // equivalent to the lifetime of the Module, the value will remain valid
        unsafe { FunctionReference::new(self, *value, function.r#type) }
    }

    /// # Panics
    /// This function can panic if the `name` cannot be converted into a `CString`
    pub fn define_global(
        &mut self,
        name: &str,
        r#type: &dyn Type,
        value: Option<&ConstValue>,
    ) -> GlobalId {
        let id = GlobalId(self.id, self.symbols.intern(name));

        let name = CString::from_str(name).unwrap();
        // SAFETY: the module reference, type and name are all valid pointers for the duration of
        // the call
        let global = unsafe { LLVMAddGlobal(self.reference, r#type.as_llvm_ref(), name.as_ptr()) };
        // SAFETY: We just created the global, and the value must be correct
        unsafe {
            LLVMSetInitializer(
                global,
                value.map_or_else(|| LLVMGetUndef(r#type.as_llvm_ref()), Value::as_llvm_ref),
            );
        };

        self.globals.insert(id, global);

        id
    }

    fn build_global_initializers(&mut self) {
        if self.global_initializers.is_empty() {
            return;
        }

        let global = GLOBAL_INITIALIZERS_ENTRY_TYPE.with(|r#type| {
            let initializers_array_type = types::Array::new(r#type, self.global_initializers.len());

            let initializer_values: Vec<_> = self
                .global_initializers
                .iter()
                .map(|x| {
                    (
                        x.priority,
                        self.get_function(x.function).as_value(),
                        x.initialized_data_pointer.as_ref(),
                    )
                })
                .map(|x| InitializersEntryType::const_values(&x.0.into(), &x.1, x.2))
                .collect();

            self.define_global(
                "llvm.global_ctors",
                &initializers_array_type,
                Some(&initializers_array_type.const_values(&initializer_values)),
            )
        });

        // SAFETY: The global was just crated, it's valid
        unsafe {
            LLVMSetLinkage(
                *self.globals.get(&global).unwrap(),
                LLVMLinkage::LLVMAppendingLinkage,
            );
        };
    }

    /// # Panics
    /// If the provided ID is invalid. This most likely means a bug, since globals cannot be in any
    /// way removed.
    pub fn get_global(&self, id: GlobalId) -> ConstValue {
        let result = *self.globals.get(&id).unwrap();

        // SAFETY: the global is connected to the current module, so it is valid
        unsafe { ConstValue::new(result) }
    }
}

impl Drop for ModuleBuilder {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: if `reference` is not null, we own the module and are free to dispose it
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
