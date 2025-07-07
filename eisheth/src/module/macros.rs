#[macro_export]
macro_rules! define_function_caller {
    ($name:ident, (runtime $(($($argument_name:ident : $argument_type:ty),*))?)) => {
        paste::paste! {
            pub fn $name<$($([<T $argument_name>]),*)?>(
                &self,
                i: &::eisheth::function::instruction_builder::InstructionBuilder,
                $($($argument_name : [<T $argument_name>]),*)?
            )
                $(where $(::eisheth::value::ConstOrDynamicValue: From<[<T $argument_name>]>),* )?
            {
                let _ = i.direct_call(
                    self. $name,
                    &[$($($argument_name .into()),*)?], ""
                );
            }
        }
    };
    ($name:ident, (runtime $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)) => {
        paste::paste! {
            pub fn $name<$($([<T $argument_name>]),*)?>(
                &self,
                i: &::eisheth::function::instruction_builder::InstructionBuilder,
                $($($argument_name : [<T $argument_name>]),*)?
            ) -> ::eisheth::value::DynamicValue
                $(where $(::eisheth::value::ConstOrDynamicValue: From<[<T $argument_name>]>),* )?
            {
                i.direct_call(
                    self. $name,
                    &[$($($argument_name .into()),*)?], stringify!($name)
                )
            }
        }
    }
}

#[macro_export]
macro_rules! define_function {
    ($module:expr, $name:ident, (runtime $(($($argument_name:ident : $argument_type:ty),*))?)) => {
        ::eisheth::define_function!($module, $name, (runtime $(($($argument_name : $argument_type),*))? -> ()))
    };
    ($module:expr, $name:ident, (runtime $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)) => {
        // SAFETY: Signatures of the functions match
        let $name = unsafe {
            $module.define_runtime_function(
                &::eisheth::function::declaration::FunctionSignature::new(
                    stringify!($name),
                    ::eisheth::types::Function::new(
                        &<($return_type) as ::eisheth::types::RepresentedAs>::representation(),
                        &[$($(&<($argument_type) as ::eisheth::types::RepresentedAs>::representation()),*)?],
                    ),
                    // TODO allow creating internal functions as well
                    ::eisheth::function::declaration::Visibility::Export,
                ),
                runtime:: $name as (unsafe extern "C" fn($($($argument_type),*)?) -> $return_type) as usize,
            )
        };
    };
}

#[macro_export]
macro_rules! define_module {
    (module $name:ident {
        $( $function_name:ident : $function_contents:tt; )*
    }) => {
        pub struct Definition {
            $($function_name: ::eisheth::module::DeclaredFunctionDescriptor),*
        }

        impl Definition {
            pub fn import_into(&self, module: &mut ::eisheth::module::builder::ModuleBuilder) -> ImportedDefinition {
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
            $(::eisheth::define_function!(module, $function_name, $function_contents);)*

            Definition {
                $($function_name),*
            }
        }

        pub struct ImportedDefinition {
            $($function_name: ::eisheth::module::DeclaredFunctionDescriptor),*
        }

        impl ImportedDefinition {
            $(::eisheth::define_function_caller!($function_name, $function_contents);)*
        }
    };
}
