use eisheth::define_module;

define_module! {
    module side {
        side_fn : (builder () -> u64);
    }
}

mod builder {
    use eisheth::{function::builder::FunctionBuilder, value::ConstValue};

    pub(super) fn side_fn(function: &FunctionBuilder) {
        let block = function.create_block("entry");

        let result: ConstValue = 7u64.into();
        block.build(|i| i.r#return(result));
    }
}
