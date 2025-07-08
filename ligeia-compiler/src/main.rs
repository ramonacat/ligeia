use eisheth::{
    module::builder::ModuleBuilder,
    types::{RepresentedAs, TypeExtensions},
};
mod value;
mod vector;

use eisheth::{
    function::declaration::{FunctionSignature, Visibility},
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
    types::{self},
    value::ConstValue,
};

use crate::{value::ffi::Value, vector::ffi::Vector};

fn main() {
    let mut package_builder = PackageBuilder::new();

    let side_module = package_builder.add_module("side").unwrap();
    let side = side_module.define_function(
        &FunctionSignature::new(
            "side_fn",
            types::Function::new(&u64::representation(), &[]),
            Visibility::Export,
        ),
        |function| {
            let block = function.create_block("entry");

            let result: ConstValue = 7u64.into();
            block.build(|i| i.r#return(result));
        },
    );

    let value_definition = value::define(&mut package_builder);
    let value_vector_definition = vector::define(&mut package_builder);

    let main_module = package_builder.add_module("main").unwrap();

    // TODO: some nice interface so the imports are more readable?
    let vector_definition_in_main = value_vector_definition.import_into(main_module);
    let value_definition_in_main = value_definition.import_into(main_module);

    let types = Vector::with_type(|r#type| main_module.define_global("types", r#type, None));
    let types = main_module.get_global(types);

    let type_value: ConstValue = 1u64.into();
    let test_type = main_module.define_global("type", &u64::representation(), Some(&type_value));
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

    let side = main_module.import_function(side).unwrap();
    let other = main_module.define_function(
        &FunctionSignature::new(
            "other",
            types::Function::new(&u64::representation(), &[&u64::representation()]),
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
    let main_function = main_module.define_function(
        &FunctionSignature::new(
            "main",
            types::Function::new(&u64::representation(), &[&u64::representation()]),
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
                let value_from_side = i.direct_call(side, &[], "cross_module");
                let sum3 = i.add(sum2, value_from_side, "cross_module_sum");

                i.r#return(sum3)
            });
        },
    );

    let package = match package_builder.build() {
        Ok(package) => package,
        Err(errors) => {
            panic!("Failed to build the modules:\n{errors}");
        }
    };

    for (module_name, raw_ir) in package.ir_per_module() {
        println!("IR for {module_name}:\n{raw_ir}");
    }

    println!("Final linked IR:\n{}", package.final_ir());

    let jit = Jit::new(package).unwrap();

    // SAFETY: The signature matches the signature of the declaration
    let callable: JitFunction<unsafe extern "C" fn(u64) -> u64> =
        unsafe { jit.get_function(main_function) };

    // SAFETY: The JITted code is correct and memory safe, right? I'm sure there aren't any bugs
    // lurking
    let result = unsafe { callable.call(12) };

    println!("Result: {result}");
}

fn install_types_initializer(
    main_module: &mut ModuleBuilder,
    vector_definition_in_main: &vector::ImportedDefinition,
    value_definition_in_main: &value::ImportedDefinition,
    types: ConstValue,
    test_type: ConstValue,
) {
    // TODO we should be pointing to the initialized data here (i.e. None should be Some(types))
    main_module.define_global_initializer("types", 0, None, |function| {
        let entry = function.create_block("entry");
        entry.build(|i| {
            Value::with_type(|r#type| {
                vector_definition_in_main.initializer(&i, types, r#type.sizeof());
            });

            let pointer = vector_definition_in_main.push_uninitialized(&i, types);
            value_definition_in_main.initialize_pointer(&i, pointer, test_type);

            value_definition_in_main.debug_print(&i, pointer);

            let pointer = vector_definition_in_main.push_uninitialized(&i, types);
            value_definition_in_main.initialize_pointer(&i, pointer, test_type);

            value_definition_in_main.debug_print(&i, pointer);

            let pointer = vector_definition_in_main.push_uninitialized(&i, types);
            value_definition_in_main.initialize_pointer(&i, pointer, test_type);

            value_definition_in_main.debug_print(&i, pointer);

            i.return_void()
        });
    });
}
