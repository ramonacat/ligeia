use eisheth::{types::RepresentedAs, value::ConstValue};
mod ffi;

use eisheth::{
    function::{
        declaration::{FunctionSignature, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types::{self, Type},
    value::DynamicValue,
};

use crate::vector::ffi::Vector;

// TODO we should also allow for defining a vector of Opaque type, where only the FFI-side code
// undrerstands the type inside
// TODO add some debug_print method, that prints the contents of the vector
pub fn define(package_builder: &mut PackageBuilder, element_type: &dyn Type) -> Definition {
    let module = package_builder.add_module("vector").unwrap();

    let initializer = module.define_function(
        &FunctionSignature::new(
            "vector_initializer",
            types::Function::new(&<()>::representation(), &[&<*mut Vector>::representation()]),
            Visibility::Export,
        ),
        |f| {
            let entry = f.create_block("entry");

            let vector = f.get_argument(0).unwrap();

            entry.build(|i| {
                let memory_pointer = Vector::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 0, "memory_pointer")
                        .unwrap()
                });

                let length: ConstValue = 1u64.into();
                let memory = i.malloc_array(element_type, &length, "memory");
                i.store(&memory_pointer, &memory);

                let capacity_pointer = Vector::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 1, "capacity_pointer")
                        .unwrap()
                });
                let capacity: ConstValue = 1u32.into();
                i.store(&capacity_pointer, &capacity);

                let length_pointer = Vector::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 2, "length_pointer")
                        .unwrap()
                });
                let length: ConstValue = 0u32.into();
                i.store(&length_pointer, &length);

                i.r#return(None)
            });
        },
    );

    // SAFETY: The signature of the rust-side function matches the one in the FFI-side declaration
    let push_uninitialized = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "push_uninitialized",
                types::Function::new(
                    &<*mut u8>::representation(),
                    &[&<*mut Vector>::representation()],
                ),
                Visibility::Export,
            ),
            runtime::push_uninitialized as unsafe extern "C" fn(*mut Vector) -> *mut u8 as usize,
        )
    };

    Definition {
        initializer,
        push_uninitialized,
    }
}

pub struct Definition {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
}

impl Definition {
    pub(crate) fn import_into(&self, module: &mut ModuleBuilder) -> ImportedDefinition {
        let initializer = module.import_function(self.initializer).unwrap();
        let push_uninitialized = module.import_function(self.push_uninitialized).unwrap();

        ImportedDefinition {
            initializer,
            push_uninitialized,
        }
    }
}

// TODO we should have element type here probably and validate wherever possible?
pub struct ImportedDefinition {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
}

impl ImportedDefinition {
    pub(crate) fn initialize(&self, i: &InstructionBuilder, pointer: &dyn eisheth::value::Value) {
        let _ = i.direct_call(self.initializer, &[pointer], "");
    }

    #[allow(unused)]
    pub(crate) fn push_uninitialized(
        &self,
        i: &InstructionBuilder,
        vector: &dyn eisheth::value::Value,
    ) -> DynamicValue {
        i.direct_call(self.push_uninitialized, &[vector], "uninitilized_element")
    }
}

impl Type for ImportedDefinition {
    fn as_llvm_ref(&self) -> eisheth::llvm_sys::prelude::LLVMTypeRef {
        Vector::with_type(Type::as_llvm_ref)
    }
}

mod runtime {
    use crate::vector::ffi::Vector;

    // TODO: This should do an actual push, i.e. add to length and do a realloc and increase
    // capacity if needed
    pub(super) unsafe extern "C" fn push_uninitialized(vector: *mut Vector) -> *mut u8 {
        // SAFETY: It's up to the user to ensure that `vector` is a valid pointer and that they
        // won't use the returned refernce for longer than the `vector` is valid
        (unsafe { &*vector }).data
    }
}
