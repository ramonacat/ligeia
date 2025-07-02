use eisheth::{
    function::{
        declaration::{FunctionDeclarationDescriptor, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types::{self, Type},
    value::{ConstValue, Value},
};

pub struct Definition {
    r#type: types::Struct,
    initializer: DeclaredFunctionDescriptor,
}

impl Definition {
    pub(crate) fn import_into(&self, module: &mut ModuleBuilder) -> ImportedDefinition {
        let initializer = module.import_function(self.initializer).unwrap();

        ImportedDefinition {
            initializer,
            r#type: &self.r#type,
        }
    }
}

pub struct ImportedDefinition<'definition> {
    r#type: &'definition types::Struct,
    initializer: DeclaredFunctionDescriptor,
}

impl ImportedDefinition<'_> {
    pub(crate) const fn r#type(&self) -> &types::Struct {
        self.r#type
    }

    pub(crate) fn const_null(&self) -> ConstValue {
        self.r#type.const_value(&[
            types::Pointer.const_uninitialized().unwrap(),
            types::U32.const_uninitialized().unwrap(),
            types::U32.const_uninitialized().unwrap(),
        ])
    }

    pub(crate) fn initialize(&self, i: &InstructionBuilder, pointer: &dyn Value) {
        i.direct_call(self.initializer, &[pointer], "");
    }
}

pub fn define(package_builder: &mut PackageBuilder) -> Definition {
    let module = package_builder.add_module("vector").unwrap();

    let r#type = types::Struct::new(
        "vector",
        vec![
            Box::new(types::Pointer),
            Box::new(types::U32),
            Box::new(types::U32),
        ],
    );

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
                let memory_pointer = r#type
                    .get_field_pointer(&i, &vector, 0, "memory_pointer")
                    .unwrap();
                let memory = i.malloc_array(&types::U64, &types::U64::const_value(1), "memory");
                i.store(&memory_pointer, &memory);

                let capacity_pointer = r#type
                    .get_field_pointer(&i, &vector, 1, "capacity_pointer")
                    .unwrap();
                let capacity = types::U32::const_value(1);
                i.store(&capacity_pointer, &capacity);

                let length_pointer = r#type
                    .get_field_pointer(&i, &vector, 1, "length_pointer")
                    .unwrap();
                let length = types::U32::const_value(2);
                i.store(&length_pointer, &length);

                i.r#return(None)
            });
        },
    );

    Definition {
        r#type,
        initializer,
    }
}
