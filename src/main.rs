#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::undocumented_unsafe_blocks,
    warnings
)]

use llvm::{
    jit::{Jit, function::JitFunction},
    module::Module,
    types,
};

mod llvm;

fn main() {
    let main_module = Module::new("main");

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

                i.r#return(&sum)
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
