use std::marker::PhantomData;

pub struct JitFunction<TFunction> {
    pointer: usize,
    _phantom: PhantomData<TFunction>,
}

impl<TFunction> JitFunction<TFunction> {
    pub(super) const fn new(pointer: usize) -> Self {
        Self {
            pointer,
            _phantom: PhantomData,
        }
    }
}

macro_rules! jit_function_impl {
    ($($argument:tt),*) => {
        #[allow(unused)]
        impl<
            TReturn,
            $($argument),*
        > JitFunction<
            unsafe extern "C" fn ($($argument),*) -> TReturn
        > {
            /// # Safety
            /// The function signature on the Rust side must match the jitted function, and the
            /// function itself must be memory-safe
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub unsafe fn call(
                &self,
                $(paste::paste!([<$argument _arg>]): $argument),*
            ) -> TReturn {
                let callable:unsafe extern "C" fn ($($argument),*) -> TReturn =
                    // SAFETY: The caller has ensured that the signature is correct and the pointer
                    // can be safely transmuted
                    unsafe { std::mem::transmute(self.pointer) };

                // SAFETY: The caller has ensured that the signature matches and that the pointed
                // at code is memory-safe
                unsafe { callable($(paste::paste!([<$argument _arg>])),*) }
            }
        }
    };
}

impl<TReturn> JitFunction<unsafe extern "C" fn() -> TReturn> {
    /// # Safety
    /// The function must be memory safe, and the signature on the Rust side must match the jitted
    /// function's signature.
    #[must_use]
    pub unsafe fn call(&self) -> TReturn {
        let callable: unsafe extern "C" fn() -> TReturn =
            // SAFETY: The caller has ensured that the signature matches
            unsafe { std::mem::transmute(self.pointer) };

        // SAFETY: The caller ensured that the code pointed to is correct, and the arguments are as
        // well
        unsafe { callable() }
    }
}

jit_function_impl!(TArg1);
jit_function_impl!(TArg1, TArg2);
jit_function_impl!(TArg1, TArg2, TArg3);
jit_function_impl!(TArg1, TArg2, TArg3, TArg4);
jit_function_impl!(TArg1, TArg2, TArg3, TArg4, TArg5);
jit_function_impl!(TArg1, TArg2, TArg3, TArg4, TArg5, TArg6);
jit_function_impl!(TArg1, TArg2, TArg3, TArg4, TArg5, TArg6, TArg7);
jit_function_impl!(TArg1, TArg2, TArg3, TArg4, TArg5, TArg6, TArg7, TArg8);
