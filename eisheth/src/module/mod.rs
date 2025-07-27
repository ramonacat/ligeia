pub mod builder;
pub mod built;

use std::{ffi::CStr, marker::PhantomData};

use llvm_sys::{
    core::{LLVMDisposeMessage, LLVMPrintModuleToString},
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{global_symbol::GlobalSymbol, package::id::PackageId, types::function::Function};
use crate::{
    Visibility,
    types::OpaqueType,
    value::{ConstOrDynamicValue, ConstValue, ValueReference},
};

pub(crate) trait AnyModule {
    fn as_llvm_ref(&self) -> LLVMModuleRef;
}

pub(crate) trait AnyModuleExtensions {
    fn dump_ir(&self) -> String;
}

impl<T: AnyModule> AnyModuleExtensions for T {
    fn dump_ir(&self) -> String {
        // SAFETY: We own the reference, so it's valid until drop
        let raw_string = unsafe { LLVMPrintModuleToString(self.as_llvm_ref()) };

        assert!(!raw_string.is_null());

        // SAFETY: LLVM will always return a valid string
        let result = unsafe { CStr::from_ptr(raw_string).to_str().unwrap().to_string() };

        // SAFETY: There are no more references to the raw_string, we're good to free it
        unsafe { LLVMDisposeMessage(raw_string) };

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(PackageId, GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct DeclaredFunctionDescriptor {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: Function,
    visibility: Visibility,
}

impl DeclaredFunctionDescriptor {
    pub(crate) const fn name(&self) -> GlobalSymbol {
        self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclaredGlobalDescriptor {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: OpaqueType,
    visibility: Visibility,
}

impl ValueReference for DeclaredGlobalDescriptor {
    fn value(&self, module: &builder::ModuleBuilder) -> ConstOrDynamicValue {
        module.get_global(*self).into()
    }
}

impl DeclaredGlobalDescriptor {
    #[must_use]
    pub const fn r#type(&self) -> OpaqueType {
        self.r#type
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalReference<'module> {
    _module: PhantomData<&'module dyn AnyModule>,
    reference: LLVMValueRef,
    r#type: OpaqueType,
}

impl GlobalReference<'_> {
    #[must_use]
    pub const fn r#type(&self) -> OpaqueType {
        self.r#type
    }
}

impl From<GlobalReference<'_>> for ConstValue {
    fn from(val: GlobalReference<'_>) -> Self {
        // SAFETY: we kept the reference to the module, so it must still be live, which means the
        // global exists
        unsafe { Self::new(val.reference) }
    }
}

impl From<GlobalReference<'_>> for ConstOrDynamicValue {
    fn from(value: GlobalReference<'_>) -> Self {
        let value: ConstValue = value.into();

        value.into()
    }
}

impl ValueReference for GlobalReference<'_> {
    fn value(&self, _module: &builder::ModuleBuilder) -> ConstOrDynamicValue {
        (*self).into()
    }
}
