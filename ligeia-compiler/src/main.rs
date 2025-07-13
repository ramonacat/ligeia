mod test_program;
mod value;
mod vector;

use eisheth::{
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
};

fn main() {
    let mut package_builder = PackageBuilder::new();

    let side = test_program::side::define(&mut package_builder);
    let value = value::define(&mut package_builder);
    let vector = vector::define(&mut package_builder);
    let exported_globals = test_program::exported_globals::define(&mut package_builder);
    let imports = test_program::imports::define(&mut package_builder, &value);

    let main_function = test_program::main::define(
        &mut package_builder,
        &side,
        &value,
        &vector,
        &exported_globals,
        &imports,
    )
    .into_freestanding()
    .get_main();

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
