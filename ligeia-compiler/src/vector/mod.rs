mod ffi;

use std::marker::PhantomData;

use eisheth::{
    function::{
        declaration::{FunctionDeclarationDescriptor, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types::{self, Type},
    value::Value,
};

use crate::vector::ffi::Vector;

pub fn define<T>(package_builder: &mut PackageBuilder) -> Definition<T> {
    let module = package_builder.add_module("vector").unwrap();

    let initializer = module.define_function(
        &FunctionDeclarationDescriptor::new(
            "vector_initializer",
            types::Function::new(&types::Void, &[&types::Pointer]),
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
                let memory = i.malloc_array(&types::U64, &types::U64::const_value(1), "memory");
                i.store(&memory_pointer, &memory);

                let capacity_pointer = Vector::<T>::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 1, "capacity_pointer")
                        .unwrap()
                });
                let capacity = types::U32::const_value(1);
                i.store(&capacity_pointer, &capacity);

                let length_pointer = Vector::<T>::with_type(|r#type| {
                    r#type
                        .get_field_pointer(&i, &vector, 1, "length_pointer")
                        .unwrap()
                });
                let length = types::U32::const_value(2);
                i.store(&length_pointer, &length);

                i.r#return(None)
            });
        },
    );

    Definition {
        initializer,
        _items: PhantomData,
    }
}

pub struct Definition<T> {
    initializer: DeclaredFunctionDescriptor,
    _items: PhantomData<*mut T>,
}

impl<T> Definition<T> {
    pub(crate) fn import_into(&self, module: &mut ModuleBuilder) -> ImportedDefinition<T> {
        let initializer = module.import_function(self.initializer).unwrap();

        ImportedDefinition {
            initializer,
            _items: PhantomData,
        }
    }
}

// TODO we should have element type here probably and validate wherever possible?
pub struct ImportedDefinition<T> {
    initializer: DeclaredFunctionDescriptor,
    _items: PhantomData<*mut T>,
}

impl<T> ImportedDefinition<T> {
    pub(crate) fn initialize(&self, i: &InstructionBuilder, pointer: &dyn Value) {
        i.direct_call(self.initializer, &[pointer], "");
    }
}

impl<T> Type for ImportedDefinition<T> {
    fn as_llvm_ref(&self) -> eisheth::llvm_sys::prelude::LLVMTypeRef {
        Vector::<T>::with_type(Type::as_llvm_ref)
    }

    fn const_uninitialized(&self) -> Option<eisheth::value::ConstValue> {
        Vector::<T>::with_type(Type::const_uninitialized)
    }
}
