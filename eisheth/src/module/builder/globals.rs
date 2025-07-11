use std::{ffi::CString, str::FromStr as _};

use llvm_sys::{
    core::{LLVMAddGlobal, LLVMGetUndef, LLVMSetInitializer, LLVMSetLinkage},
    prelude::LLVMValueRef,
};

use crate::{
    function::declaration::Visibility,
    module::{DeclaredGlobalDescriptor, builder::ModuleBuilder},
    types::Type,
    value::{ConstValue, Value},
};

pub fn define_global<T: Type>(
    module: &ModuleBuilder,
    visibility: Visibility,
    name: &str,
    r#type: T,
    value: Option<&ConstValue>,
) -> (DeclaredGlobalDescriptor, LLVMValueRef) {
    let interned_name = module.symbols.intern(name);

    let name = CString::from_str(name).unwrap();
    // SAFETY: the module reference, type and name are all valid pointers for the duration of
    // the call
    let global = unsafe { LLVMAddGlobal(module.reference, r#type.as_llvm_ref(), name.as_ptr()) };
    // SAFETY: We just created the global, and the value must be correct
    unsafe {
        LLVMSetInitializer(
            global,
            value.map_or_else(|| LLVMGetUndef(r#type.as_llvm_ref()), Value::as_llvm_ref),
        );
    };
    // SAFETY: The global was just created and is valid
    unsafe {
        LLVMSetLinkage(
            global,
            match visibility {
                Visibility::Internal => llvm_sys::LLVMLinkage::LLVMInternalLinkage,
                Visibility::Export => llvm_sys::LLVMLinkage::LLVMExternalLinkage,
            },
        );
    };

    let descriptor = DeclaredGlobalDescriptor {
        module_id: module.id,
        name: interned_name,
        r#type: r#type.into(),
        visibility,
    };

    (descriptor, global)
}
