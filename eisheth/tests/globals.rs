use eisheth::{jit::Jit, package::builder::PackageBuilder};

mod test_module {
    use eisheth::define_module;

    define_module!(
        module test_module {
            internal global value : u64;
            init_value : builder (^value);
            get_value : builder (^value) -> u64;

            global_initializer : 0, init_value, value;
        }
    );

    mod builder {
        use eisheth::{
            function::builder::FunctionBuilder, module::DeclaredGlobalDescriptor,
            types::RepresentedAs, value::ConstValue,
        };

        pub(super) fn init_value(function: &FunctionBuilder, value: DeclaredGlobalDescriptor) {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let magic: ConstValue = 1234u64.into();
                i.store(&value, &magic);
                i.return_void()
            });
        }

        pub(super) fn get_value(function: &FunctionBuilder, value: DeclaredGlobalDescriptor) {
            let entry = function.create_block("entry");

            entry.build(|i| {
                let loaded = i.load(&value, u64::representation(), "value");

                i.r#return(loaded)
            });
        }
    }
}

#[test]
pub fn initialize_global() {
    let mut package_builder = PackageBuilder::new();
    let module = test_module::define(&mut package_builder).into_freestanding();

    let function = module.get_get_value();
    let package = package_builder.build().unwrap();

    let jit = Jit::new(package.into_package()).unwrap();
    let func = unsafe { jit.get_function::<unsafe extern "C" fn() -> u64>(function) };

    assert_eq!(1234, unsafe { func.call() });
}
