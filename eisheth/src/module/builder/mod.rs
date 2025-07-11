use crate::{
    context::diagnostic::{DIAGNOSTIC_HANDLER, DiagnosticHandler},
    global_symbol::GlobalSymbol,
    module::{
        AnyModule, AnyModuleExtensions, DeclaredGlobalDescriptor, GlobalReference,
        builder::{
            errors::{FunctionImportError, ModuleBuildError},
            global_finalizers::{
                FinalizersEntryType, GLOBAL_FINALIZER_TYPE, GLOBAL_FINALIZERS_ENTRY_TYPE,
                GlobalFinalizerDescriptor,
            },
            global_initializers::{GLOBAL_INITIALIZER_TYPE, GlobalInitializerDescriptor},
        },
    },
};

pub mod errors;
mod functions;
mod global_finalizers;
mod global_initializers;
mod globals;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    hash::Hash,
    rc::Rc,
    str::FromStr as _,
};

use llvm_sys::{
    LLVMLinkage,
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMDisposeMessage, LLVMDisposeModule, LLVMModuleCreateWithNameInContext, LLVMSetLinkage,
    },
    prelude::{LLVMModuleRef, LLVMValueRef},
};

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
    value::ConstValue,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct GlobalId(ModuleId, GlobalSymbol);

pub struct ModuleBuilder {
    id: ModuleId,
    reference: LLVMModuleRef,
    symbols: Rc<GlobalSymbols>,
    global_initializers: Vec<GlobalInitializerDescriptor>,
    global_finalizers: Vec<GlobalFinalizerDescriptor>,
    global_mappings: HashMap<String, usize>,
    global_values: HashMap<DeclaredGlobalDescriptor, LLVMValueRef>,
    function_values: HashMap<DeclaredFunctionDescriptor, LLVMValueRef>,
}

impl AnyModule for ModuleBuilder {
    fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }
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
            global_initializers: vec![],
            global_finalizers: vec![],
            global_mappings: HashMap::new(),
            global_values: HashMap::new(),
            function_values: HashMap::new(),
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
        let (id, function) = functions::declare_function(self, declaration);

        self.function_values.insert(id, function);
        self.global_mappings
            .insert(declaration.name().to_string(), runtime_function_address);

        id
    }

    // TODO: some cute macro so that these functions are easier to define?
    pub fn define_function(
        &mut self,
        declaration: &FunctionSignature,
        implement: impl FnOnce(&FunctionBuilder),
    ) -> DeclaredFunctionDescriptor {
        let (id, function) = functions::define_function(self, declaration, implement);

        self.function_values.insert(id, function);

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

    pub fn define_global_finalizer(
        &mut self,
        name: &str,
        priority: u32,
        finalized_data_pointer: Option<ConstValue>,
        implement: impl FnOnce(&FunctionBuilder),
    ) {
        let function = GLOBAL_FINALIZER_TYPE.with(|initializer| {
            self.define_function(
                &FunctionSignature::new(
                    format!("global_finalizer_{name}"),
                    *initializer,
                    Visibility::Internal,
                ),
                implement,
            )
        });
        self.global_finalizers.push(GlobalFinalizerDescriptor {
            priority,
            function,
            finalized_data_pointer,
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
        let (id, function) = functions::import_function(self, id)?;

        self.function_values.insert(id, function);

        Ok(id)
    }

    pub(crate) fn build(mut self) -> Result<Module, ModuleBuildError> {
        self.build_global_initializers();
        self.build_global_finalizers();

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
            let message = unsafe { CStr::from_ptr(out_message) }
                .to_str()
                .unwrap()
                .to_string();

            // SAFETY: We made a copy of the message, this pointer won't be used anymore
            unsafe {
                LLVMDisposeMessage(out_message);
            };

            let diagnostics = DIAGNOSTIC_HANDLER.with(DiagnosticHandler::take_diagnostics);

            return Err(ModuleBuildError {
                module_name: self.symbols.resolve(self.id.1),
                message,
                diagnostics,
                raw_ir: self.dump_ir(),
            });
        }

        if !out_message.is_null() {
            // SAFETY: We've checked that the message was set to something, so it must be a valid
            // string
            let message = unsafe { CStr::from_ptr(out_message).to_str().unwrap().to_string() };

            // SAFETY: This pointer won't be used anymore, safe to dispose
            unsafe { LLVMDisposeMessage(out_message) };

            // TODO we should probably return the message to the user instead of just printing
            eprintln!("{message}");
        }

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        let mut functions = HashMap::new();
        std::mem::swap(&mut functions, &mut self.function_values);

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
        let value = self.function_values.get(&function).unwrap();

        // SAFETY: The functions here were transfered from the ModuleBuilder, so we know they
        // belong to this module, so as long as the function reference has a life time at least
        // equivalent to the lifetime of the Module, the value will remain valid
        unsafe { FunctionReference::new(self, *value, function.r#type) }
    }

    /// # Panics
    /// This function can panic if the `name` cannot be converted into a `CString`
    pub fn define_global<T: Type>(
        &mut self,
        name: &str,
        r#type: T,
        value: Option<&ConstValue>,
    ) -> DeclaredGlobalDescriptor {
        let (descriptor, global) = globals::define_global(self, name, r#type, value);
        self.global_values.insert(descriptor, global);

        descriptor
    }

    fn build_global_initializers(&mut self) {
        if self.global_initializers.is_empty() {
            return;
        }

        let global = GLOBAL_INITIALIZERS_ENTRY_TYPE.with(|r#type| {
            let initializers_array_type =
                types::Array::new(*r#type, self.global_initializers.len());

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
                initializers_array_type,
                Some(&initializers_array_type.const_values(&initializer_values)),
            )
        });

        // SAFETY: The global was just crated, it's valid
        unsafe {
            LLVMSetLinkage(
                *self.global_values.get(&global).unwrap(),
                LLVMLinkage::LLVMAppendingLinkage,
            );
        };
    }

    fn build_global_finalizers(&mut self) {
        if self.global_finalizers.is_empty() {
            return;
        }

        let global = GLOBAL_FINALIZERS_ENTRY_TYPE.with(|r#type| {
            let finalizers_array_type = types::Array::new(*r#type, self.global_finalizers.len());

            let finalizer_values: Vec<_> = self
                .global_finalizers
                .iter()
                .map(|x| {
                    (
                        x.priority,
                        self.get_function(x.function).as_value(),
                        x.finalized_data_pointer.as_ref(),
                    )
                })
                .map(|x| FinalizersEntryType::const_values(&x.0.into(), &x.1, x.2))
                .collect();

            self.define_global(
                "llvm.global_dtors",
                finalizers_array_type,
                Some(&finalizers_array_type.const_values(&finalizer_values)),
            )
        });

        // SAFETY: The global was just crated, it's valid
        unsafe {
            LLVMSetLinkage(
                *self.global_values.get(&global).unwrap(),
                LLVMLinkage::LLVMAppendingLinkage,
            );
        };
    }

    /// # Panics
    /// If the provided ID is invalid. This most likely means a bug, since globals cannot be in any
    /// way removed.
    #[must_use]
    pub fn get_global(&self, id: DeclaredGlobalDescriptor) -> GlobalReference<'_> {
        let result = *self.global_values.get(&id).unwrap();

        // SAFETY: the global is connected to the current module, so it is valid
        GlobalReference {
            _module: std::marker::PhantomData,
            reference: result,
            r#type: id.r#type,
        }
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
