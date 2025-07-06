use eisheth::ffi_struct;

#[ffi_struct]
#[repr(C)]
#[allow(unused)]
// TODO Add methods for safe operations instead of making fields pub?
pub struct Vector<T> {
    pub data: *mut T,
    length: u32,
    capacity: u32,
}
