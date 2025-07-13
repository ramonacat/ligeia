use eisheth::define_module;

use crate::vector::ffi::Vector;

define_module!(
    module main import (super::side, crate::value, crate::vector, super::exported_globals, super::imports) {
        internal global types : Vector;
        internal global test_type : u64 = 1;

        types_initializer : builder (
            ^vector.initializer,
            ^vector.push_uninitialized,
            ^value.initialize_pointer,
            ^value.debug_print,
            ^types,
            ^test_type,
        );
        types_finalizer : builder (^vector.finalizer, ^types);
        internal other : builder (value: u64) -> u64;
        main : builder (^imports.show_info, ^exported_globals.important_number, ^side.side_fn, ^other, input: u64) -> u64;

        global_initializer : 1024, types_initializer, types;
        global_finalizer : 1024, types_finalizer, types;
    }
);

mod builder {
    use eisheth::{
        function::builder::FunctionBuilder,
        module::{DeclaredFunctionDescriptor, DeclaredGlobalDescriptor},
        types::{RepresentedAs as _, TypeExtensions as _},
        value::{ConstValue, DynamicValue},
    };

    use crate::value::ffi::Value;

    pub(super) fn main(
        function: &FunctionBuilder,
        show_info: DeclaredFunctionDescriptor,
        important_number: DeclaredGlobalDescriptor,
        side_fn: DeclaredFunctionDescriptor,
        other: DeclaredFunctionDescriptor,
        arg0: DynamicValue,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let _ = i.direct_call(show_info, &[], "");

            let base: ConstValue = 32u64.into();
            let sum = i.add(&base, &arg0, "add");
            let arg: ConstValue = 2u64.into();
            let value_from_other = i.direct_call(other, &[&arg], "calling_other");
            let sum2 = i.add(&sum, &value_from_other, "add_again");
            let value_from_side = i.direct_call(side_fn, &[], "side_fn");
            let sum3 = i.add(&sum2, &value_from_side, "cross_module_sum");
            let important_number_value = i.load(
                &important_number,
                important_number.r#type(),
                "important_number",
            );
            let sum4 = i.add(&sum3, &important_number_value, "imported_global_sum");

            i.r#return(sum4)
        });
    }

    pub(super) fn other(function: &FunctionBuilder<'_>, _arg: DynamicValue) {
        let block = function.create_block("entry");

        block.build(|i| {
            let left: ConstValue = 2u64.into();
            let right: ConstValue = 11u64.into();
            let sum = i.add(&left, &right, "sum");

            i.r#return(sum)
        });
    }

    pub(super) fn types_finalizer(
        function: &FunctionBuilder,
        finalizer: DeclaredFunctionDescriptor,
        types: DeclaredGlobalDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let _ = i.direct_call(finalizer, &[&types], "");

            i.return_void()
        });
    }

    pub(super) fn types_initializer(
        function: &FunctionBuilder<'_>,
        initializer: DeclaredFunctionDescriptor,
        push_unitialized: DeclaredFunctionDescriptor,
        initialize_pointer: DeclaredFunctionDescriptor,
        debug_print: DeclaredFunctionDescriptor,
        types: DeclaredGlobalDescriptor,
        test_type: DeclaredGlobalDescriptor,
    ) {
        let entry = function.create_block("entry");
        entry.build(|i| {
            let _ = i.direct_call(
                initializer,
                &[&types, &Value::representation().sizeof()],
                "",
            );

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
    }
}
