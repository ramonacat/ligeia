#[macro_export]
macro_rules! make_function_signature {
    ($name:ident, $(($($argument_name:ident : $argument_type:ty),*))?) => {
        make_function_signature!($name, ($(($($argument_name : $argument_type),*))? -> ()));
    };
    ($name:ident, $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty) => {
        &::eisheth::function::declaration::FunctionSignature::new(
            stringify!($name),
            ::eisheth::types::Function::new(
                &<($return_type) as ::eisheth::types::RepresentedAs>::representation(),
                &[$($(&<($argument_type) as ::eisheth::types::RepresentedAs>::representation()),*)?],
            ),
            // TODO allow creating internal functions as well
            ::eisheth::function::declaration::Visibility::Export,
        )
    };
}

#[macro_export]
macro_rules! define_module_function_caller {
    (
        $name:ident,
        (runtime $(($($argument_name:ident : $argument_type:ty),*))?)
    ) => {
        ::eisheth::define_module_function_caller!(
            @__impl $name,
            ($(($($argument_name : $argument_type),*))? -> ()),
            _,
            "",
        );
    };
    (
        $name:ident,
        (runtime $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)
    ) => {
        ::eisheth::define_module_function_caller!(
            @__impl $name,
            ($(($($argument_name : $argument_type),*))? -> ::eisheth::value::DynamicValue),
            result,
            stringify!($name),
            result
        );
    };
    (
        $name:ident,
        (builder $(($($argument_name:ident : $argument_type:ty),*))?)
    ) => {
        ::eisheth::define_module_function_caller!(
            @__impl $name,
            ($(($($argument_name : $argument_type),*))? -> ()),
            _,
            "",
        );
    };
    (
        $name:ident,
        (builder $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)
    ) => {
        ::eisheth::define_module_function_caller!(
            @__impl $name,
            ($(($($argument_name : $argument_type),*))? -> ::eisheth::value::DynamicValue),
            result,
            stringify!($name),
            result
        );
    };
    (
        @__impl $name:ident,
        ($(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty),
        $let_name:expr,
        $binding_name:expr,
        $($return_statement:expr)?
    ) => {
        paste::paste! {
            pub fn $name<$($([<T $argument_name>]),*)?>(
                &self,
                i: &::eisheth::function::instruction_builder::InstructionBuilder,
                $($($argument_name : [<T $argument_name>]),*)?
            ) -> $return_type
                $(where $(::eisheth::value::ConstOrDynamicValue: From<[<T $argument_name>]>),* )?
            {
                let $let_name = i.direct_call(
                    self. $name,
                    &[$($($argument_name .into()),*)?],
                    $binding_name
                );

                $($return_statement)?
            }
        }
    };
}

#[macro_export]
macro_rules! define_module_function {
    (
        $module:expr,
        $name:ident,
        (runtime $(($($argument_name:ident : $argument_type:ty),*))?)
    ) => {
        ::eisheth::define_module_function!($module, $name, (runtime $(($($argument_name : $argument_type),*))? -> ()))
    };
    (
        $module:expr,
        $name:ident,
        (runtime $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)
    ) => {
        // SAFETY: Signatures of the functions match
        let $name = unsafe {
            $module.define_runtime_function(
                ::eisheth::make_function_signature!($name, $(($($argument_name : $argument_type),*))? -> $return_type),
                runtime:: $name as (unsafe extern "C" fn($($($argument_type),*)?) -> $return_type) as usize,
            )
        };
    };
    (
        $module:expr,
        $name:ident,
        (builder $(($($argument_name:ident : $argument_type:ty),*))?)
    ) => {
        ::eisheth::define_module_function!($module, $name, (runtime $(($($argument_name : $argument_type),*))? -> ()))
    };
    (
        $module:expr,
        $name:ident,
        (builder $(($($argument_name:ident : $argument_type:ty),*))? -> $return_type:ty)
    ) => {
        // TODO the signature generation is repeated, new macro needed to handle it?
        let $name = $module.define_function(
            ::eisheth::make_function_signature!($name, $(($($argument_name : $argument_type),*))? -> $return_type),
            |function| {
                let mut i:usize = 0;
                builder::$name(
                    function,
                    $(
                        $(function.get_argument({
                            // This is so we have a reference to an argument in the repetition
                            let _$argument_name = ();
                            i += 1;

                            i - 1
                        })),*
                    )?
                );
            }
        );
    }
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
