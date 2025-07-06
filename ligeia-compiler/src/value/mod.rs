pub mod ffi;

use eisheth::{
    function::{
        declaration::{FunctionDeclarationDescriptor, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types,
};

use crate::value::ffi::Value;

// TODO add some debug_print method that will print the value for debug purposes
pub fn define(package_builder: &mut PackageBuilder) -> ValueDefinition {
    let module = package_builder.add_module("value").unwrap();

    // SAFETY: The function signature on Rust side matches the FFI-side
    let initialize_pointer = unsafe {
        module.define_runtime_function(
            &FunctionDeclarationDescriptor::new(
                "initialize_pointer",
                types::Function::new(&types::Void, &[&types::Pointer, &types::Pointer]),
                Visibility::Export,
            ),
            runtime::initialize_pointer as unsafe extern "C" fn(*mut Value, *mut u8) as usize,
        )
    };

    ValueDefinition { initialize_pointer }
}

pub struct ValueDefinition {
    initialize_pointer: DeclaredFunctionDescriptor,
}

impl ValueDefinition {
    pub fn import_into(&self, module: &mut ModuleBuilder) -> ImportedValueDefinition {
        let initialize_pointer = module.import_function(self.initialize_pointer).unwrap();

        ImportedValueDefinition { initialize_pointer }
    }
}

pub struct ImportedValueDefinition {
    initialize_pointer: DeclaredFunctionDescriptor,
}

impl ImportedValueDefinition {
    pub fn initialize_pointer(
        &self,
        i: &InstructionBuilder,
        value: &dyn eisheth::value::Value,
        target: &dyn eisheth::value::Value,
    ) {
        i.direct_call(self.initialize_pointer, &[value, target], "");
    }
}

mod runtime {
    use crate::value::ffi::{Type, Value};

    pub(super) unsafe extern "C" fn initialize_pointer(value: *mut Value, target_pointer: *mut u8) {
        // SAFETY: It's up to the user to provide a a valid pointer to a value and a valid
        // target_pointer. As long as those are correct, the created Value will be valid
        unsafe {
            (*value).r#type = Type::Pointer;
            (*value).data = target_pointer as u64;
        }
    }
}
