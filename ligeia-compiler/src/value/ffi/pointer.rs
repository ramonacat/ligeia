use std::fmt::Debug;

use crate::value::ffi::{Type, Value};

#[repr(transparent)]
pub struct PointerValue(Value);

impl Debug for PointerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ptr(0x{:0>16X})", self.0.data)
    }
}

impl PointerValue {
    pub unsafe fn initialize(value: *mut Value, target: *mut u8) -> *mut Self {
        // SAFETY: it's up to the caller to provide valid pointers, as long as those are right, the
        // value will initialize correctly
        unsafe {
            (*value).r#type = Type::Pointer;
            (*value).data = target as u64;
        }

        value.cast()
    }

    pub(crate) unsafe fn ptr_from(value: *mut Value) -> Option<*mut Self> {
        // SAFETY: it's the caller's responsibility to provide a valid pointer
        if (unsafe { &*value }).r#type == Type::Pointer {
            return Some(value.cast());
        }

        None
    }
}
