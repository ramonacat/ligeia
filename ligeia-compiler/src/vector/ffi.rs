use eisheth::ffi_struct;

#[ffi_struct]
#[repr(C)]
#[derive(Debug)]
pub struct Vector {
    data: *mut u8,
    length: u32,
    capacity: u32,
    element_size: u32,
}

impl Vector {
    pub(crate) fn initialize(value: *mut Self, element_size: u64) {
        // SAFETY: The caller must provide a valid pointer
        let pointer = unsafe { &mut *value };
        pointer.element_size = u32::try_from(element_size).unwrap();
        // SAFETY: The caller must give us a valid, aligned, non-zero element_size already set
        pointer.data = unsafe { libc::malloc(pointer.element_size as usize) }.cast();
        pointer.length = 0;
        pointer.capacity = 1;
    }

    pub(crate) fn push_uninitialized(vector: *mut Self) -> *mut u8 {
        // SAFETY: the user must pass a pointer to a valid vector
        let vector = unsafe { &mut *vector };

        if vector.length + 1 > vector.capacity {
            // SAFETY: we know the new size is bigger than previous and aligned, because
            // element_size must be
            vector.data = unsafe {
                libc::realloc(
                    vector.data.cast(),
                    vector.capacity as usize * vector.element_size as usize * 2usize,
                )
            }
            .cast();
            assert!(!vector.data.is_null());
            vector.capacity *= 2;
        }

        vector.length += 1;

        // SAFETY: We've ensured that there's enough memory for the element_size, and the calleer
        // expects it to be uninitialized
        unsafe {
            vector.data.byte_offset(
                isize::try_from(vector.length - 1).unwrap()
                    * isize::try_from(vector.element_size).unwrap(),
            )
        }
    }

    pub(crate) fn finalize(vector: *mut Self) {
        // SAFETY: The caller must ensure they call us with a valid pointer, and that the vector
        // will not be used anymore
        unsafe {
            libc::free((&mut *vector).data.cast());
            (&mut *vector).data = std::ptr::null_mut();
        };
    }
}
