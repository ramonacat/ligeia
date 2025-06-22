#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::undocumented_unsafe_blocks,
    warnings
)]

use llvm::{jit::Jit, module::Module, types};

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

                // TODO we should require a return here, the funcs that do terminators should
                // return some TerminatorToken or something, to avoid creating avoidably incorrect
                // IR
                i.r#return(&sum);
            });
        },
    );

    let built_module = main_module.build();

    let jit = Jit::new(built_module);

    // TODO would be cool to have a way to refer to functions by some reference, but the change of
    // Module into BuiltModule is making that kinda hard
    // SAFETY: The signature matches the signature of the declaration, and (hopefully) the JITted
    // code is safe
    let callable: extern "C" fn(u64) -> u64 =
        unsafe { std::mem::transmute(jit.get_function("main")) };

    let result = callable(12);

    println!("Result: {result}");
}
