use llvm::{
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
    types,
};

mod llvm;

fn main() {
    let mut package_builder = PackageBuilder::new();

    let side_module = package_builder.add_module("side");
    let side = side_module.define_function(
        "side_fn",
        types::Function::new(&types::U64, &[]),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| i.r#return(&types::U64::const_value(7)));
        },
    );

    let main_module = package_builder.add_module("main");
    let side = main_module.import_function(side);
    let other = main_module.define_function(
        "other",
        types::Function::new(&types::U64, &[&types::U64]),
        |function| {
            let block = function.create_block("entry");

            block.build(|i| {
                let left = types::U64::const_value(2);
                let right = types::U64::const_value(11);
                let sum = i.add(&left, &right, "sum");

                i.r#return(&sum)
            });
        },
    );
    let main_function = main_module.define_function(
        "main",
        types::Function::new(&types::U64, &[&types::U64]),
        |function| {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let base = types::U64::const_value(32);
                let sum = i.add(&base, &function.get_argument(0).unwrap(), "add");
                let value_from_other =
                    i.direct_call(other, &[types::U64::const_value(2)], "calling_other");
                let sum2 = i.add(&sum, &value_from_other, "add_again");
                let value_from_side = i.direct_call(side, &[], "cross_module");
                let sum3 = i.add(&sum2, &value_from_side, "cross_module_sum");

                i.r#return(&sum3)
            });
        },
    );

    let package = match package_builder.build() {
        Ok(package) => package,
        Err(error) => {
            eprintln!("{error}");

            return;
        }
    };
    let jit = Jit::new(package);

    // SAFETY: The signature matches the signature of the declaration
    // TODO can we use a FunctionId here instead of the name?
    let callable: JitFunction<unsafe extern "C" fn(u64) -> u64> =
        unsafe { jit.get_function(main_function) };

    // SAFETY: The JITted code is correct and memory safe, right? I'm sure there aren't any bugs
    // lurking
    let result = unsafe { callable.call(12) };

    println!("Result: {result}");
}
