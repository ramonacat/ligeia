mod vector;

use eisheth::{
    function::declaration::{FunctionDeclarationDescriptor, Visibility},
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
    types,
};

fn main() {
    let mut package_builder = PackageBuilder::new();

    let side_module = package_builder.add_module("side").unwrap();
    let side = side_module.define_function(
        &FunctionDeclarationDescriptor::new(
            "side_fn",
            types::Function::new(&types::U64, &[]),
            Visibility::Export,
        ),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| i.r#return(Some(&types::U64::const_value(7))));
        },
    );

    let vector_definition = vector::define(&mut package_builder);

    let main_module = package_builder.add_module("main").unwrap();

    let imported_defintion = vector_definition.import_into(main_module);

    let test_vector = main_module.define_global(
        "test vector",
        imported_defintion.r#type(),
        &imported_defintion.const_null(),
    );

    main_module.define_global_initializer("init_test_vector", |function| {
        let entry = function.create_block("entry");
        entry.build(|i| {
            imported_defintion.initialize(&i, &test_vector);

            i.r#return(None)
        });
    });

    let side = main_module.import_function(side).unwrap();
    let other = main_module.define_function(
        &FunctionDeclarationDescriptor::new(
            "other",
            types::Function::new(&types::U64, &[&types::U64]),
            Visibility::Internal,
        ),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| {
                let left = types::U64::const_value(2);
                let right = types::U64::const_value(11);
                let sum = i.add(&left, &right, "sum");

                i.r#return(Some(&sum))
            });
        },
    );
    let main_function = main_module.define_function(
        &FunctionDeclarationDescriptor::new(
            "main",
            types::Function::new(&types::U64, &[&types::U64]),
            Visibility::Export,
        ),
        move |function| {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let base = types::U64::const_value(32);
                let sum = i.add(&base, &function.get_argument(0).unwrap(), "add");
                let value_from_other =
                    i.direct_call(other, &[&types::U64::const_value(2)], "calling_other");
                let sum2 = i.add(&sum, &value_from_other, "add_again");
                let value_from_side = i.direct_call(side, &[], "cross_module");
                let sum3 = i.add(&sum2, &value_from_side, "cross_module_sum");

                i.r#return(Some(&sum3))
            });
        },
    );

    let package = match package_builder.build() {
        Ok(package) => package,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }

            panic!("failed to build modlues");
        }
    };
    let jit = Jit::new(package);

    // SAFETY: The signature matches the signature of the declaration
    let callable: JitFunction<unsafe extern "C" fn(u64) -> u64> =
        unsafe { jit.get_function(main_function) };

    // SAFETY: The JITted code is correct and memory safe, right? I'm sure there aren't any bugs
    // lurking
    let result = unsafe { callable.call(12) };

    println!("Result: {result}");
}
