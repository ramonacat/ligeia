use eisheth::define_module;

use crate::value::ffi::Value;
pub mod ffi;

define_module! {
    module value {
        initialize_pointer : (runtime (value: *mut Value, pointer: *mut u8));
        debug_print : (runtime (value: *mut Value));
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
