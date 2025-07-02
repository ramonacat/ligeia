use eisheth::{
    function::{
        declaration::{FunctionDeclarationDescriptor, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{FunctionDeclaration, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types::{self, value::ConstValue},
};

pub struct Definition {
    r#type: types::Struct,
    initializer: FunctionDeclaration,
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
    initializer: FunctionDeclaration,
}

impl ImportedDefinition<'_> {
    pub(crate) const fn r#type(&self) -> &types::Struct {
        self.r#type
    }

    pub(crate) fn const_null(&self) -> ConstValue {
        self.r#type.const_value(&[
            types::Pointer::const_null(),
            types::U32::const_value(0),
            types::U32::const_value(0),
        ])
    }

    pub(crate) fn initialize(&self, i: &InstructionBuilder, pointer: &dyn types::value::Value) {
        i.direct_call(self.initializer, &[pointer], "");
    }
}

pub fn define(package_builder: &mut PackageBuilder) -> Definition {
    let module = package_builder.add_module("vector").unwrap();

    let r#type = types::Struct::new("vector", &[&types::Pointer, &types::U32, &types::U32]);

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
                let memory_pointer = r#type.get_field_pointer(&i, &vector, 0, "memory_pointer");
                let memory = i.malloc_array(&types::U64, &types::U64::const_value(1), "memory");
                i.store(&memory_pointer, &memory);

                let capacity_pointer = r#type.get_field_pointer(&i, &vector, 1, "capacity_pointer");
                let capacity = types::U32::const_value(1);
                i.store(&capacity_pointer, &capacity);

                let length_pointer = r#type.get_field_pointer(&i, &vector, 1, "length_pointer");
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
