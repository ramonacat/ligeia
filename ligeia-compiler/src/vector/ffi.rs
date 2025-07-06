use eisheth::ffi_struct;

#[ffi_struct]
#[repr(C)]
#[allow(unused)]
// TODO Add methods for safe operations instead of making fields pub?
pub struct Vector {
    pub data: *mut u8,
    length: u32,
    capacity: u32,
}
