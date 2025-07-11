use eisheth::{
    function::declaration::{FunctionSignature, Visibility},
    module::DeclaredFunctionDescriptor,
    package::builder::PackageBuilder,
    types::{self, RepresentedAs},
    value::ConstValue,
};

use crate::{
    install_types_initializer,
    test_program::side,
    value,
    vector::{self, ffi::Vector},
};

pub fn define(package_builder: &mut PackageBuilder) -> DeclaredFunctionDescriptor {
    let side_definition = side::define(package_builder);
    let value_definition = value::define(package_builder);
    let value_vector_definition = vector::define(package_builder);

    let main_module = package_builder.add_module("main").unwrap();

    // TODO: some nice interface so the imports are more readable?
    let vector_definition_in_main = value_vector_definition.import_into(main_module);
    let value_definition_in_main = value_definition.import_into(main_module);
    let side_definition_in_main = side_definition.import_into(main_module);

    let types = main_module.define_global("types", Vector::r#type(), None);
    let types = main_module.get_global(types);

    let type_value: ConstValue = 1u64.into();
    let test_type = main_module.define_global("type", u64::representation(), Some(&type_value));
    let test_type = main_module.get_global(test_type);

    install_types_initializer(
        main_module,
        &vector_definition_in_main,
        &value_definition_in_main,
        types,
        test_type,
    );

    // TODO: set the finalized_data_pointer to point at types
    main_module.define_global_finalizer("types", 0, None, |function| {
        let entry = function.create_block("entry");
        entry.build(|i| {
            vector_definition_in_main.finalizer(&i, types);

            i.return_void()
        });
    });

    let other = main_module.define_function(
        &FunctionSignature::new(
            "other",
            types::Function::new(u64::representation(), &[u64::representation().into()]),
            Visibility::Internal,
        ),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| {
                let left: ConstValue = 2u64.into();
                let right: ConstValue = 11u64.into();
                let sum = i.add(left, right, "sum");

                i.r#return(sum)
            });
        },
    );

    main_module.define_function(
        &FunctionSignature::new(
            "main",
            types::Function::new(u64::representation(), &[u64::representation().into()]),
            Visibility::Export,
        ),
        move |function| {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let base: ConstValue = 32u64.into();
                let sum = i.add(base, function.get_argument(0).unwrap(), "add");
                let arg: ConstValue = 2u64.into();
                let value_from_other = i.direct_call(other, &[arg.into()], "calling_other");
                let sum2 = i.add(sum, value_from_other, "add_again");
                let value_from_side = side_definition_in_main.side_fn(&i);
                let sum3 = i.add(sum2, value_from_side, "cross_module_sum");

                i.r#return(sum3)
            });
        },
    )
}
