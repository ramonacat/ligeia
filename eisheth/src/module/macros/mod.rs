#[macro_export]
macro_rules! define_module {
    (module $name:ident {
        $( $function_name:ident : $function_contents:tt; )*
    }) => {
        pub struct Definition {
            $($function_name: ::eisheth::module::DeclaredFunctionDescriptor),*
        }

        impl Definition {
            pub fn import_into(
                &self,
                module: &mut ::eisheth::module::builder::ModuleBuilder
            ) -> ImportedDefinition {
                $(
                    let $function_name = module.import_function(self.$function_name).unwrap();
                )*

                ImportedDefinition {
                    $($function_name,)*
                }
            }
        }

        pub fn define(
            package_builder: &mut ::eisheth::package::builder::PackageBuilder
        ) -> Definition {
            let module = package_builder.add_module(stringify!($name)).unwrap();
            $(::eisheth::define_module_function!(module, $function_name, $function_contents);)*

            Definition {
                $($function_name),*
            }
        }

        pub struct ImportedDefinition {
            $($function_name: ::eisheth::module::DeclaredFunctionDescriptor),*
        }

        impl ImportedDefinition {
            $(::eisheth::define_module_function_caller!($function_name, $function_contents);)*
        }
    };
}
