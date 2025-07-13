use eisheth::{
    function::declaration::{FunctionSignature, Visibility},
    module::DeclaredFunctionDescriptor,
    package::builder::PackageBuilder,
    types::{self, RepresentedAs},
    value::{ConstOrDynamicValue, ConstValue},
};

use crate::{
    install_types_initializer,
    test_program::{exported_globals, imports, side},
    value,
    vector::{self, ffi::Vector},
};

pub fn define(package_builder: &mut PackageBuilder) -> DeclaredFunctionDescriptor {
    let side_definition = side::define(package_builder);
    let value_definition = value::define(package_builder);
    let value_vector_definition = vector::define(package_builder);
    let exported_globals_definition = exported_globals::define(package_builder);
    let imports_definition = imports::define(package_builder, &value_definition);

    let main_module = package_builder.add_module("main").unwrap();

    // TODO: some nice interface so the imports are more readable?
    let vector_definition_in_main = value_vector_definition.import_into(main_module);
    let value_definition_in_main = value_definition.import_into(main_module);
    let side_definition_in_main = side_definition.import_into(main_module);
    let exported_globals_in_main = exported_globals_definition.import_into(main_module);
    let imports_in_main = imports_definition.import_into(main_module);

    let types = main_module.define_global(
        Visibility::Internal,
        "types",
        Vector::representation(),
        None,
    );

    let type_value: ConstValue = 1u64.into();
    let test_type = main_module.define_global(
        Visibility::Internal,
        "type",
        u64::representation(),
        Some(&type_value),
    );

    install_types_initializer(
        main_module,
        &vector_definition_in_main,
        &value_definition_in_main,
        types,
        test_type,
    );

    let types: ConstOrDynamicValue = main_module.get_global(types).into();
    let types_finalizer = main_module.define_function(
        &FunctionSignature::new(
            "types_finalizer",
            types::Function::new(<() as RepresentedAs>::representation(), &[]),
            Visibility::Export,
        ),
        |function| {
            let entry = function.create_block("entry");
            entry.build(|i| {
                let _ = i.direct_call(vector_definition_in_main.get_finalizer(), &[&types], "");

                i.return_void()
            });
        },
    );
    // TODO: set the finalized_data_pointer to point at types
    main_module.define_global_finalizer(0, None, types_finalizer);

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
                let sum = i.add(&left, &right, "sum");

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
                let show_info = imports_in_main.get_show_info();
                let _ = i.direct_call(show_info, &[], "");

                let important_number = exported_globals_in_main.get_important_number();
                let base: ConstValue = 32u64.into();
                let sum = i.add(&base, &function.get_argument(0).unwrap(), "add");
                let arg: ConstValue = 2u64.into();
                let value_from_other = i.direct_call(other, &[&arg], "calling_other");
                let sum2 = i.add(&sum, &value_from_other, "add_again");
                let value_from_side =
                    i.direct_call(side_definition_in_main.get_side_fn(), &[], "side_fn");
                let sum3 = i.add(&sum2, &value_from_side, "cross_module_sum");
                let important_number_value = i.load(
                    &important_number,
                    important_number.r#type(),
                    "important_number",
                );
                let sum4 = i.add(&sum3, &important_number_value, "imported_global_sum");

                i.r#return(sum4)
            });
        },
    )
}
