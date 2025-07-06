use eisheth::{ffi_enum, ffi_struct};

#[ffi_enum]
#[repr(u8)]
#[allow(unused)]
pub enum Type {
    Empty = 0,

    U32 = 3,
    U64 = 4,

    Pointer = 64,
}

#[ffi_struct]
#[repr(C)]
// TODO Instead of making fields pub, should we instead add methods that safely manimpulate the
// value?
// TODO Pointers should have a type id that points at the type table, which contains the pointed-to
// type
pub struct Value {
    r#type: Type,
    unused1: u8,
    unused2: u16,
    unused3: u32,

    // TODO how do we handle u128 and SIMD types?
    // TODO should we have support for unions and make this an union instead?
    data: u64,
}

const _: () = assert!(size_of::<Value>() == 16);

impl Value {
    pub unsafe fn initialize_pointer(value: *mut Self, target: *mut u8) {
        // SAFETY: it's up to the caller to provide valid pointers, as long as those are right, the
        // value will initialize correctly
        unsafe {
            (*value).r#type = Type::Pointer;
            (*value).data = target as u64;
        }
    }
}
