use eisheth::types::RepresentedAs;
mod ffi;

use std::{marker::PhantomData, mem::MaybeUninit};

use eisheth::{
    function::{
        declaration::{FunctionDeclarationDescriptor, Visibility},
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
// TODO we should probably get rid of the Rust generic and only rely on r#type, to allow actually
// dynamically generating code that does vectors?
// TODO add some debug_print method, that prints the contents of the vector
pub fn define<T>(package_builder: &mut PackageBuilder, element_type: &dyn Type) -> Definition<T> {
    let module = package_builder.add_module("vector").unwrap();

    let initializer = module.define_function(
        &FunctionDeclarationDescriptor::new(
            "vector_initializer",
            types::Function::new(&types::Void, &[&<*mut Vector<T>>::representation()]),
            Visibility::Export,
        ),
        |f| {
            let entry = f.create_block("entry");

            let vector = f.get_argument(0).unwrap();

            entry.build(|i| {
                let memory_pointer = Vector::<T>::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 0, "memory_pointer")
                        .unwrap()
                });

                // TODO impl Into<ConstValue> for u*?
                let memory = i.malloc_array(
                    element_type,
                    &u64::representation().const_value(1),
                    "memory",
                );
                i.store(&memory_pointer, &memory);

                let capacity_pointer = Vector::<T>::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 1, "capacity_pointer")
                        .unwrap()
                });
                let capacity = u32::representation().const_value(1);
                i.store(&capacity_pointer, &capacity);

                let length_pointer = Vector::<T>::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 2, "length_pointer")
                        .unwrap()
                });
                let length = u32::representation().const_value(0);
                i.store(&length_pointer, &length);

                i.r#return(None)
            });
        },
    );

    // SAFETY: The signature of the rust-side function matches the one in the FFI-side declaration
    let push_uninitialized = unsafe {
        module.define_runtime_function(
            &FunctionDeclarationDescriptor::new(
                "push_uninitialized",
                types::Function::new(
                    &<*mut MaybeUninit<T>>::representation(),
                    &[&<*mut T>::representation()],
                ),
                Visibility::Export,
            ),
            runtime::push_uninitialized as unsafe extern "C" fn(*mut Vector<T>) -> *mut T as usize,
        )
    };

    Definition {
        initializer,
        push_uninitialized,
        _items: PhantomData,
    }
}

pub struct Definition<T> {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
    _items: PhantomData<*mut T>,
}

impl<T> Definition<T> {
    pub(crate) fn import_into(&self, module: &mut ModuleBuilder) -> ImportedDefinition<T> {
        let initializer = module.import_function(self.initializer).unwrap();
        let push_uninitialized = module.import_function(self.push_uninitialized).unwrap();

        ImportedDefinition {
            initializer,
            push_uninitialized,
            _items: PhantomData,
        }
    }
}

// TODO we should have element type here probably and validate wherever possible?
pub struct ImportedDefinition<T> {
    initializer: DeclaredFunctionDescriptor,
    push_uninitialized: DeclaredFunctionDescriptor,
    _items: PhantomData<*mut T>,
}

impl<T> ImportedDefinition<T> {
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

impl<T> Type for ImportedDefinition<T> {
    fn as_llvm_ref(&self) -> eisheth::llvm_sys::prelude::LLVMTypeRef {
        Vector::<T>::with_type(Type::as_llvm_ref)
    }
}

mod runtime {
    use crate::vector::ffi::Vector;

    // TODO: This should do an actual push, i.e. add to length and do a realloc and increase
    // capacity if needed
    pub(super) unsafe extern "C" fn push_uninitialized<T>(vector: *mut Vector<T>) -> *mut T {
        // SAFETY: It's up to the user to ensure that `vector` is a valid pointer and that they
        // won't use the returned refernce for longer than the `vector` is valid
        (unsafe { &*vector }).data
    }
}
