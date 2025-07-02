mod global_initializers;

use std::{
    collections::HashMap, error::Error, ffi::CString, fmt::Display, rc::Rc, str::FromStr as _,
};

use llvm_sys::{
    LLVMLinkage,
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMDisposeModule, LLVMDumpModule,
        LLVMModuleCreateWithNameInContext, LLVMSetInitializer, LLVMSetLinkage,
    },
    prelude::{LLVMModuleRef, LLVMValueRef},
};
use thiserror::Error;

use super::{DeclaredFunctionDescriptor, ModuleId, built::Module};
use crate::{
    LLVM_CONTEXT,
    function::{
        builder::{FunctionBuilder, FunctionReference},
        declaration::{FunctionDeclarationDescriptor, Visibility},
    },
    global_symbol::GlobalSymbols,
    module::builder::global_initializers::GLOBAL_INITIALIZERS_ENTRY_TYPE,
    package::context::PackageContext,
    types::{
        self, Type,
        value::{ConstValue, Value as _},
    },
};

thread_local! {
    pub(super) static GLOBAL_INITIALIZER_TYPE: types::Function = types::Function::new(&types::Void, &[]);
}

#[derive(Debug)]
pub struct ModuleBuildError {
    module_name: String,
    message: String,
}

impl Display for ModuleBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to build the module \"{}\":\n{}",
            self.module_name, self.message
        )
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

pub struct ModuleBuilder {
    id: ModuleId,
    reference: LLVMModuleRef,
    symbols: Rc<GlobalSymbols>,
    functions: HashMap<DeclaredFunctionDescriptor, LLVMValueRef>,
    global_initializers: Vec<DeclaredFunctionDescriptor>,
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
        }
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }

    pub fn define_function(
        &mut self,
        declaration: &FunctionDeclarationDescriptor,
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
        implement: impl FnOnce(&FunctionBuilder),
    ) {
        let function = GLOBAL_INITIALIZER_TYPE.with(|initializer| {
            self.define_function(
                &FunctionDeclarationDescriptor::new(
                    format!("global_initializer_{name}"),
                    *initializer,
                    Visibility::Internal,
                ),
                implement,
            )
        });
        self.global_initializers.push(function);
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

            return Err(ModuleBuildError {
                module_name: self.symbols.resolve(self.id.1),
                message,
            });
        }
        // SAFETY: We have a valid, non-null `reference`, so this function can't fail
        unsafe { LLVMDumpModule(self.reference) };

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        let mut functions = HashMap::new();
        std::mem::swap(&mut functions, &mut self.functions);

        let mut global_initializers = vec![];
        std::mem::swap(&mut global_initializers, &mut self.global_initializers);

        // SAFETY: We have ensured that the reference is not owned by this current object
        Ok(unsafe {
            Module::new(
                self.id,
                reference,
                functions,
                self.symbols.clone(),
                global_initializers,
            )
        })
    }

    pub(crate) fn get_function(&self, function: DeclaredFunctionDescriptor) -> FunctionReference {
        let value = self.functions.get(&function).unwrap();

        // SAFETY: The functions here were transfered from the ModuleBuilder, so we know they
        // belong to this module, so as long as the function reference has a life time at least
        // equivalent to the lifetime of the Module, the value will remain valid
        unsafe { FunctionReference::new(self, *value, function.r#type) }
    }

    /// # Panics
    /// This function can panic if the `name` cannot be converted into a `CString`
    pub fn define_global(&self, name: &str, r#type: &dyn Type, value: &ConstValue) -> ConstValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: the module reference, type and name are all valid pointers for the duration of
        // the call
        let global = unsafe { LLVMAddGlobal(self.reference, r#type.as_llvm_ref(), name.as_ptr()) };

        // SAFETY: We just created the global, and the value must be correct
        unsafe { LLVMSetInitializer(global, value.as_llvm_ref()) };

        // SAFETY: We just created the global, and it will not ever be destroyed
        unsafe { ConstValue::new(global) }
    }

    fn build_global_initializers(&self) {
        if self.global_initializers.is_empty() {
            return;
        }

        let global = GLOBAL_INITIALIZERS_ENTRY_TYPE.with(|r#type| {
            let initializers_array_type = types::Array::new(r#type, self.global_initializers.len());

            self.define_global(
                "llvm.global_ctors",
                &initializers_array_type,
                &initializers_array_type.const_uninitialized().unwrap(),
            )
        });

        // SAFETY: The global was just crated, it's valid
        unsafe { LLVMSetLinkage(global.as_llvm_ref(), LLVMLinkage::LLVMAppendingLinkage) };
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
