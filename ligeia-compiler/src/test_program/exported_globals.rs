use eisheth::define_module;

define_module! {
    module exported_globals {
        global important_number : u64;
        init_important_number : builder (^important_number);
        fini_important_number : builder (^important_number);
        global_initializer : 1024, init_important_number, important_number;
        global_finalizer : 1024, fini_important_number, important_number;
    }
}

mod builder {
    use eisheth::{
        function::builder::FunctionBuilder, module::DeclaredGlobalDescriptor, value::ConstValue,
    };

    pub(super) fn init_important_number(
        function: &FunctionBuilder,
        important_number: DeclaredGlobalDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let value: ConstValue = 1024u64.into();
            i.store(&important_number, &value);

            i.return_void()
        });
    }

    pub(super) fn fini_important_number(
        function: &FunctionBuilder,
        important_number: DeclaredGlobalDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let value: ConstValue = 0u64.into();
            i.store(&important_number, &value);

            i.return_void()
        });
    }
}
