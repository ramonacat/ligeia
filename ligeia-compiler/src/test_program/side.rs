use eisheth::define_module;

define_module! {
    module side {
        internal global number : u64;
        internal secret : builder (^number, input: u64) -> u64;
        side_fn : builder (^secret, ^number) -> u64;
    }
}

mod builder {
    use eisheth::{
        function::builder::FunctionBuilder,
        module::{DeclaredFunctionDescriptor, DeclaredGlobalDescriptor},
        value::{ConstValue, DynamicValue},
    };

    pub(super) fn secret(
        function: &FunctionBuilder,
        number: DeclaredGlobalDescriptor,
        input: DynamicValue,
    ) {
        let block = function.create_block("entry");

        let right: ConstValue = 32u64.into();

        block.build(|i| {
            i.store(number, right);
            let sum = i.add(input, right, "sum");

            i.r#return(sum)
        });
    }

    pub(super) fn side_fn(
        function: &FunctionBuilder,
        secret: DeclaredFunctionDescriptor,
        number: DeclaredGlobalDescriptor,
    ) {
        let block = function.create_block("entry");

        let result: ConstValue = 7u64.into();
        block.build(|i| {
            let sum = i.direct_call(secret, &[result.into()], "sum");
            let number = i.load(number, number.r#type(), "number");
            let sum2 = i.add(sum, number, "sum2");

            i.r#return(sum2)
        });
    }
}
