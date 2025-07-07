use eisheth::{types::RepresentedAs, value::ConstOrDynamicValue};
pub mod ffi;

use eisheth::{
    function::{
        declaration::{FunctionSignature, Visibility},
        instruction_builder::InstructionBuilder,
    },
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    package::builder::PackageBuilder,
    types,
};

use crate::value::ffi::Value;

pub fn define(package_builder: &mut PackageBuilder) -> ValueDefinition {
    let module = package_builder.add_module("value").unwrap();

    // SAFETY: The function signature on Rust side matches the FFI-side
    let initialize_pointer = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "initialize_pointer",
                types::Function::new(
                    &<()>::representation(),
                    &[
                        &<*mut Value>::representation(),
                        &<*mut u8>::representation(),
                    ],
                ),
                Visibility::Export,
            ),
            runtime::initialize_pointer as unsafe extern "C" fn(*mut Value, *mut u8) as usize,
        )
    };

    // SAFETY: The signatures match, so this is correct
    let debug_print = unsafe {
        module.define_runtime_function(
            &FunctionSignature::new(
                "debug_print",
                types::Function::new(&<()>::representation(), &[&<*mut Value>::representation()]),
                Visibility::Export,
            ),
            runtime::debug_print as unsafe extern "C" fn(*mut Value) as usize,
        )
    };

    ValueDefinition {
        initialize_pointer,
        debug_print,
    }
}

pub struct ValueDefinition {
    initialize_pointer: DeclaredFunctionDescriptor,
    debug_print: DeclaredFunctionDescriptor,
}

impl ValueDefinition {
    pub fn import_into(&self, module: &mut ModuleBuilder) -> ImportedValueDefinition {
        let initialize_pointer = module.import_function(self.initialize_pointer).unwrap();
        let debug_print = module.import_function(self.debug_print).unwrap();

        ImportedValueDefinition {
            initialize_pointer,
            debug_print,
        }
    }
}

pub struct ImportedValueDefinition {
    initialize_pointer: DeclaredFunctionDescriptor,
    debug_print: DeclaredFunctionDescriptor,
}

impl ImportedValueDefinition {
    pub fn initialize_pointer<TValue: eisheth::value::Value, TTarget: eisheth::value::Value>(
        &self,
        i: &InstructionBuilder,
        value: TValue,
        target: TTarget,
    ) where
        ConstOrDynamicValue: From<TValue> + From<TTarget>,
    {
        let _ = i.direct_call(self.initialize_pointer, &[value.into(), target.into()], "");
    }

    pub fn debug_print<TValue: eisheth::value::Value>(&self, i: &InstructionBuilder, value: TValue)
    where
        ConstOrDynamicValue: From<TValue>,
    {
        let _ = i.direct_call(self.debug_print, &[value.into()], "");
    }
}

mod runtime {
    use crate::value::ffi::{Value, pointer::PointerValue};

    pub(super) unsafe extern "C" fn initialize_pointer(value: *mut Value, target_pointer: *mut u8) {
        // SAFETY: It's up to the user to provide a a valid pointer to a value and a valid
        // target_pointer. As long as those are correct, the created Value will be valid
        unsafe {
            PointerValue::initialize(value, target_pointer);
        }
    }

    pub(super) unsafe extern "C" fn debug_print(value: *mut Value) {
        // SAFETY: It's caller's responsibility to provide a valid pointer
        if let Some(pointer) = unsafe { PointerValue::ptr_from(value) } {
            // SAFETY: It's caller's responsibility to provide a valid pointer
            let value = unsafe { &mut *pointer };
            println!("{value:?}");
        } else {
            todo!();
        }
    }
}
