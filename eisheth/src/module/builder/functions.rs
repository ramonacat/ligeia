use std::{ffi::CString, str::FromStr as _};

use llvm_sys::{core::LLVMAddFunction, prelude::LLVMValueRef};

use crate::{
    Visibility,
    function::{builder::FunctionBuilder, declaration::FunctionSignature},
    module::{
        DeclaredFunctionDescriptor,
        builder::{ImportError, ModuleBuilder},
    },
    types::Type as _,
};

pub fn define_function(
    module: &ModuleBuilder,
    declaration: &FunctionSignature,
    implement: impl FnOnce(&FunctionBuilder),
) -> (DeclaredFunctionDescriptor, LLVMValueRef) {
    let id = DeclaredFunctionDescriptor {
        module_id: module.id,
        name: module.symbols.intern(declaration.name()),
        r#type: declaration.r#type(),
        visibility: declaration.visibility(),
    };
    let builder = FunctionBuilder::new(module, declaration);

    // TODO we should probably ask the builder to
    // verify that all blocks got built with at least a terminator
    implement(&builder);

    let function = builder.build();

    (id, function)
}

pub fn declare_function(
    module: &ModuleBuilder,
    declaration: &FunctionSignature,
) -> (DeclaredFunctionDescriptor, LLVMValueRef) {
    let id = DeclaredFunctionDescriptor {
        module_id: module.id,
        name: module.symbols.intern(declaration.name()),
        r#type: declaration.r#type(),
        visibility: declaration.visibility(),
    };

    let name = module.symbols.resolve(id.name);
    let c_name = CString::from_str(&name).unwrap();

    let function =
            // SAFETY: All the passed values come from objects which uphold guarantees about the
            // pointers being valid
            unsafe { LLVMAddFunction(module.reference, c_name.as_ptr(), id.r#type.as_llvm_ref()) };

    (id, function)
}

pub fn import_function(
    module: &ModuleBuilder,
    id: DeclaredFunctionDescriptor,
) -> Result<(DeclaredFunctionDescriptor, LLVMValueRef), ImportError> {
    if id.module_id == module.id {
        return Err(ImportError::DefinedInThisModule(
            module.symbols.resolve(id.name),
        ));
    }

    if id.visibility != Visibility::Export {
        return Err(ImportError::NotExported(module.symbols.resolve(id.name)));
    }

    let name = module.symbols.resolve(id.name);
    let c_name = CString::from_str(&name).unwrap();

    let function =
            // SAFETY: All the passed values come from objects which uphold guarantees about the
            // pointers being valid
            unsafe { LLVMAddFunction(module.reference, c_name.as_ptr(), id.r#type.as_llvm_ref()) };

    let id = DeclaredFunctionDescriptor {
        module_id: module.id,
        name: id.name,
        r#type: id.r#type,
        visibility: Visibility::Internal,
    };

    Ok((id, function))
}
