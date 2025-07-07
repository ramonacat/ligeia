use eisheth::ffi_struct;

#[ffi_struct]
#[repr(C)]
#[derive(Debug)]
// TODO Add methods for safe operations instead of making fields pub?
pub struct Vector {
    pub data: *mut u8,
    pub length: u32,
    pub capacity: u32,
    pub element_size: u32,
}
