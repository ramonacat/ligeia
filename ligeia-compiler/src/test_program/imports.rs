use eisheth::define_module;

define_module!(
    module imports import(crate::value) {
        internal global my_val : crate::value::ffi::Value;
        init_my_val : builder (^my_val, ^value.initialize_pointer);
        fini_my_val : builder (^my_val);

        show_info : builder (^my_val, ^value.debug_print);

        global_initializer: 512, init_my_val, my_val;
        global_finalizer: 512, fini_my_val, my_val;
    }
);

mod builder {
    use eisheth::{
        function::builder::FunctionBuilder,
        module::{DeclaredFunctionDescriptor, DeclaredGlobalDescriptor},
        value::ConstValue,
    };

    pub(super) fn init_my_val(
        function: &FunctionBuilder,
        my_val: DeclaredGlobalDescriptor,
        value_initialize_pointer: DeclaredFunctionDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let null: ConstValue = std::ptr::null_mut::<()>().into();
            let _ = i.direct_call(value_initialize_pointer, &[&my_val, &null], "");
            i.return_void()
        });
    }

    pub(super) fn fini_my_val(function: &FunctionBuilder, _my_val: DeclaredGlobalDescriptor) {
        let entry = function.create_block("entry");
        entry.build(|i| i.return_void());
    }

    pub(super) fn show_info(
        function: &FunctionBuilder,
        my_val: DeclaredGlobalDescriptor,
        value_debug_print: DeclaredFunctionDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let _ = i.direct_call(value_debug_print, &[&my_val], "");

            i.return_void()
        });
    }
}
