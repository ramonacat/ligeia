use llvm::{
    global_symbol::GlobalSymbols,
    jit::{Jit, function::JitFunction},
    package::PackageBuilder,
    types,
};

mod llvm;

fn main() {
    let symbols = GlobalSymbols::new();
    let mut package = PackageBuilder::new(&symbols);

    let side_module = package.add_module("side");
    let side = side_module.define_function(
        "side_fn",
        types::function::FunctionType::new(&types::integer::U64, &[]),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| i.r#return(&types::integer::U64::const_value(7)));
        },
    );

    let main_module = package.add_module("main");
    let side = main_module.import_function(side);
    let other = main_module.define_function(
        "other",
        types::function::FunctionType::new(&types::integer::U64, &[&types::integer::U64]),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| {
                let left = types::integer::U64::const_value(2);
                let right = types::integer::U64::const_value(11);
                let sum = i.add(&left, &right, "sum");

                i.r#return(&sum)
            });
        },
    );
    main_module.define_function(
        "main",
        types::function::FunctionType::new(&types::integer::U64, &[&types::integer::U64]),
        |function| {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let base = types::integer::U64::const_value(32);
                let sum = i.add(&base, &function.get_argument(0).unwrap(), "add");
                let value_from_other = i.direct_call(
                    other,
                    &[types::integer::U64::const_value(2)],
                    "calling_other",
                );
                let sum2 = i.add(&sum, &value_from_other, "add_again");
                let value_from_side = i.direct_call(side, &[], "cross_module");
                let sum3 = i.add(&sum2, &value_from_side, "cross_module_sum");

                i.r#return(&sum3)
            });
        },
    );

    let built_package = package.build();
    let jit = Jit::new(built_package);

    // SAFETY: The signature matches the signature of the declaration
    let callable: JitFunction<unsafe extern "C" fn(u64) -> u64> =
        unsafe { jit.get_function("main") };

    // SAFETY: The JITted code is correct and memory safe, right? I'm sure there aren't any bugs
    // lurking
    let result = unsafe { callable.call(12) };

    println!("Result: {result}");
}
