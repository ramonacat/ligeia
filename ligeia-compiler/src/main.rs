use eisheth::{module::builder::ModuleBuilder, types::TypeExtensions};

mod test_program;
mod value;
mod vector;

use eisheth::{
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
    value::ConstValue,
};

use crate::value::ffi::Value;

fn main() {
    let mut package_builder = PackageBuilder::new();

    let main_function = test_program::main::define(&mut package_builder);

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
