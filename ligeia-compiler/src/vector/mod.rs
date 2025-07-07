pub mod ffi;
use eisheth::define_module;
use ffi::Vector;

define_module! {
    module vector {
        initializer : (runtime (vector: *mut Vector, element_size: u64));
        push_uninitialized: (runtime (vector: *mut Vector) -> *mut u8);
        finalizer: (runtime (vector: *mut Vector));
    }
}

mod runtime {
    use crate::vector::ffi::Vector;

    pub(super) unsafe extern "C" fn initializer(pointer: *mut Vector, element_size: u64) {
        Vector::initialize(pointer, element_size);
    }

    pub(super) unsafe extern "C" fn push_uninitialized(vector: *mut Vector) -> *mut u8 {
        Vector::push_uninitialized(vector)
    }

    pub(super) unsafe extern "C" fn finalizer(vector: *mut Vector) {
        Vector::finalize(vector);
    }
}
