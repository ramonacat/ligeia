use llvm::{
    jit::{Jit, function::JitFunction},
    package::PackageBuilder,
    types,
};

mod llvm;

fn main() {
    let mut package = PackageBuilder::new();
    let mut main_module = package.add_module("main");

    let other = main_module.define_function(
        "other",
        types::function::FunctionType::new(&types::integer::U64, Box::new([])),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| i.r#return(&types::integer::U64::const_value(11)));
        },
    );

    main_module.define_function(
        "main",
        types::function::FunctionType::new(
            &types::integer::U64,
            Box::new([Box::new(types::integer::U64)]),
        ),
        |function| {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let base = types::integer::U64::const_value(32);
                let sum = i.add(
                    &base,
                    &function.get_argument::<types::integer::U64>(0).unwrap(),
                    "add",
                );

                let value_from_other = i.direct_call(other, "calling_other");

                let sum2 = i.add(&sum, &value_from_other, "add_again");

                i.r#return(&sum2)
            });
        },
    );

    let built_module = main_module.build();

    let jit = Jit::new(built_module);

    // SAFETY: The signature matches the signature of the declaration
    let callable: JitFunction<unsafe extern "C" fn(u64) -> u64> =
        unsafe { jit.get_function("main") };

    // SAFETY: The JITted code is correct and memory safe, right? I'm sure there aren't any bugs
    // lurking
    let result = unsafe { callable.call(12) };

    println!("Result: {result}");
}
