use eisheth::define_module;

define_module! {
    module side {
        internal secret : (builder (input: u64) -> u64);
        side_fn : (builder () -> u64);
    }
}

mod builder {
    use eisheth::{
        function::builder::FunctionBuilder,
        value::{ConstValue, DynamicValue},
    };

    pub(super) fn secret(function: &FunctionBuilder, input: DynamicValue) {
        let block = function.create_block("entry");

        let right: ConstValue = 32u64.into();

        block.build(|i| {
            let sum = i.add(input, right, "sum");

            i.r#return(sum)
        });
    }

    pub(super) fn side_fn(function: &FunctionBuilder) {
        let block = function.create_block("entry");

        let result: ConstValue = 7u64.into();
        block.build(|i| i.r#return(result));
    }
}
