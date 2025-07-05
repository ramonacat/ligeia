use eisheth::ffi_struct;

#[ffi_struct]
#[repr(C)]
#[allow(unused)]
pub struct Vector<T> {
    data: *mut T,
    length: u32,
    capacity: u32,
}
