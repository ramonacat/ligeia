use eisheth::{
    function::declaration::{FunctionSignature, Visibility},
    module::{DeclaredGlobalDescriptor, builder::ModuleBuilder},
    types::{self, RepresentedAs, TypeExtensions},
    value::ConstOrDynamicValue,
};

mod test_program;
mod value;
mod vector;

use eisheth::{
    jit::{Jit, function::JitFunction},
    package::builder::PackageBuilder,
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

// TODO move to test_program?
fn install_types_initializer(
    main_module: &mut ModuleBuilder,
    vector_definition_in_main: &vector::ImportedDefinition,
    value_definition_in_main: &value::ImportedDefinition,
    types: DeclaredGlobalDescriptor,
    test_type: DeclaredGlobalDescriptor,
) {
    let types: ConstOrDynamicValue = main_module.get_global(types).into();
    let test_type: ConstOrDynamicValue = main_module.get_global(test_type).into();

    // TODO we should be pointing to the initialized data here (i.e. None should be Some(types))
    let types_initializer = main_module.define_function(
        &FunctionSignature::new(
            "types_initializer",
            types::Function::new(<() as RepresentedAs>::representation(), &[]),
            Visibility::Export,
        ),
        |function| {
            let entry = function.create_block("entry");
            entry.build(|i| {
                let _ = i.direct_call(
                    vector_definition_in_main.get_initializer(),
                    &[&types, &Value::representation().sizeof()],
                    "",
                );

                let push_unitialized = vector_definition_in_main.get_push_uninitialized();
                let initialize_pointer = value_definition_in_main.get_initialize_pointer();
                let debug_print = value_definition_in_main.get_debug_print();

                let pointer = i.direct_call(push_unitialized, &[&types], "pointer");
                let _ = i.direct_call(initialize_pointer, &[&pointer, &test_type], "");
                let _ = i.direct_call(debug_print, &[&pointer], "");

                let pointer = i.direct_call(push_unitialized, &[&types], "pointer");
                let _ = i.direct_call(initialize_pointer, &[&pointer, &test_type], "");
                let _ = i.direct_call(debug_print, &[&pointer], "");

                let pointer = i.direct_call(push_unitialized, &[&types], "pointer");
                let _ = i.direct_call(initialize_pointer, &[&pointer, &test_type], "");
                let _ = i.direct_call(debug_print, &[&pointer], "");

                i.return_void()
            });
        },
    );
    main_module.define_global_initializer(0, None, types_initializer);
}
