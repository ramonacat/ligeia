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
#[allow(unused)]
pub struct Value {
    pub r#type: Type,
    unused1: u8,
    unused2: u16,
    unused3: u32,

    // TODO how do we handle u128 and SIMD types?
    // TODO should we have support for unions and make this an union instead?
    pub data: u64,
}

const _: () = assert!(size_of::<Value>() == 16);
