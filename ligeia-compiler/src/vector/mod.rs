use eisheth::{
    types::{RepresentedAs, TypeExtensions},
    value::ConstValue,
};
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

    // SAFETY: Signatures of the functions match
    let initializer_inner = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "vector_initializer_inner",
                types::Function::new(&<()>::representation(), &[&<*mut Vector>::representation()]),
                Visibility::Internal,
            ),
            runtime::initializer as unsafe extern "C" fn(*mut Vector) as usize,
        )
    };

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
                let element_size_pointer = Vector::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 3, "element_size_pointer")
                        .unwrap()
                });
                let element_size: ConstValue = element_type.sizeof();
                i.store(&element_size_pointer, &element_size);

                let _ = i.direct_call(initializer_inner, &[&vector], "");

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

    pub(super) unsafe extern "C" fn initializer(pointer: *mut Vector) {
        Vector::initialize(pointer);
    }

    pub(super) unsafe extern "C" fn push_uninitialized(vector: *mut Vector) -> *mut u8 {
        Vector::push_uninitialized(vector)
    }
}
