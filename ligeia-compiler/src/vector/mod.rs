use eisheth::types::RepresentedAs;
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

// TODO add some debug_print method, that prints the contents of the vector
// TODO add a destructor
pub fn define(package_builder: &mut PackageBuilder) -> Definition {
    let module = package_builder.add_module("vector").unwrap();

    // SAFETY: Signatures of the functions match
    let initializer = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "vector_initializer",
                types::Function::new(
                    &<()>::representation(),
                    &[&<*mut Vector>::representation(), &<u64>::representation()],
                ),
                Visibility::Export,
            ),
            runtime::initializer as unsafe extern "C" fn(*mut Vector, u64) as usize,
        )
    };

    // SAFETY: The signature of the rust-side function matches the one in the FFI-side declaration
    let push_uninitialized = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "vector_push_uninitialized",
                types::Function::new(
                    &<*mut u8>::representation(),
                    &[&<*mut Vector>::representation()],
                ),
                Visibility::Export,
            ),
            runtime::push_uninitialized as unsafe extern "C" fn(*mut Vector) -> *mut u8 as usize,
        )
    };

    // SAFETY: The signatures match between rust and FFI-side
    let finalizer = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "vector_finalizer",
                types::Function::new(&<()>::representation(), &[&<*mut Vector>::representation()]),
                Visibility::Export,
            ),
            runtime::finalizer as unsafe extern "C" fn(*mut Vector) as usize,
        )
    };

    Definition {
        initializer,
        push_uninitialized,
        finalizer,
    }
}

pub struct Definition {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
    finalizer: DeclaredFunctionDescriptor,
}

impl Definition {
    pub(crate) fn import_into(&self, module: &mut ModuleBuilder) -> ImportedDefinition {
        let initializer = module.import_function(self.initializer).unwrap();
        let push_uninitialized = module.import_function(self.push_uninitialized).unwrap();
        let finalizer = module.import_function(self.finalizer).unwrap();

        ImportedDefinition {
            initializer,
            push_uninitialized,
            finalizer,
        }
    }
}

// TODO we should have element type here probably and validate wherever possible?
pub struct ImportedDefinition {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
    finalizer: DeclaredFunctionDescriptor,
}

impl ImportedDefinition {
    pub(crate) fn initialize(
        &self,
        i: &InstructionBuilder,
        pointer: &dyn eisheth::value::Value,
        element_size: &dyn eisheth::value::Value,
    ) {
        let _ = i.direct_call(self.initializer, &[pointer, element_size], "");
    }

    pub(crate) fn push_uninitialized(
        &self,
        i: &InstructionBuilder,
        vector: &dyn eisheth::value::Value,
    ) -> DynamicValue {
        i.direct_call(self.push_uninitialized, &[vector], "uninitilized_element")
    }

    pub(crate) fn finalizer(&self, i: &InstructionBuilder, vector: &dyn eisheth::value::Value) {
        let _ = i.direct_call(self.finalizer, &[vector], "");
    }
}

impl Type for ImportedDefinition {
    fn as_llvm_ref(&self) -> eisheth::llvm_sys::prelude::LLVMTypeRef {
        Vector::with_type(Type::as_llvm_ref)
    }
}

mod runtime {
    use crate::vector::ffi::Vector;

    pub(super) unsafe extern "C" fn initializer(pointer: *mut Vector, element_size: u64) {
        Vector::initialize(pointer, element_size);
    }

    pub(super) unsafe extern "C" fn push_uninitialized(vector: *mut Vector) -> *mut u8 {
        Vector::push_uninitialized(vector)
    }

    pub(super) unsafe extern "C" fn finalizer(vector: *mut Vector) {
        Vector::finalize(vector);
    }
}
