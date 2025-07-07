pub mod builder;
pub mod built;

use std::ffi::CStr;

use llvm_sys::{
    core::{LLVMDisposeMessage, LLVMPrintModuleToString},
    prelude::LLVMModuleRef,
};

use super::{
    function::declaration::Visibility, global_symbol::GlobalSymbol, package::id::PackageId,
    types::function::Function,
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
