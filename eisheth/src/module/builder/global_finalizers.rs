use llvm_sys::{
    core::{LLVMConstStructInContext, LLVMStructType},
    prelude::LLVMTypeRef,
};

use crate::{
    context::LLVM_CONTEXT,
    module::DeclaredFunctionDescriptor,
    types::{self, RepresentedAs, Type},
    value::{ConstValue, Value},
};

thread_local! {
    pub(super) static GLOBAL_FINALIZERS_ENTRY_TYPE: FinalizersEntryType = FinalizersEntryType::new();
    pub(super) static GLOBAL_FINALIZER_TYPE: types::Function = types::Function::new(<()>::representation().into(), &[]);
}

pub(super) struct GlobalFinalizerDescriptor {
    pub priority: u32,
    pub function: DeclaredFunctionDescriptor,
    pub finalized_data_pointer: Option<ConstValue>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FinalizersEntryType(LLVMTypeRef);

impl FinalizersEntryType {
    fn new() -> Self {
        let mut finalizer_element_types = vec![
            u32::representation().as_llvm_ref(),
            <*mut fn()>::representation().as_llvm_ref(),
            <*mut u8>::representation().as_llvm_ref(),
        ];

        // SAFETY: The element types are valid for the duration of the call
        let finalizers_type = unsafe {
            LLVMStructType(
                finalizer_element_types.as_mut_ptr(),
                u32::try_from(finalizer_element_types.len()).unwrap(),
                0,
            )
        };

        Self(finalizers_type)
    }

    pub(crate) fn const_values(
        priority: &ConstValue,
        finalizer: &ConstValue,
        finalized_value: Option<&ConstValue>,
    ) -> ConstValue {
        let mut values = vec![
            priority.as_llvm_ref(),
            finalizer.as_llvm_ref(),
            finalized_value.map_or_else(
                || types::Pointer::const_null().as_llvm_ref(),
                Value::as_llvm_ref,
            ),
        ];

        // SAFETY: The values were crated from valid wrapper objects, the pointer and length are
        // valid, the context is valid
        let result = LLVM_CONTEXT.with(|context| unsafe {
            LLVMConstStructInContext(
                context.as_llvm_ref(),
                values.as_mut_ptr(),
                u32::try_from(values.len()).unwrap(),
                0,
            )
        });

        // SAFETY: We just created the value, it is correct
        unsafe { ConstValue::new(result) }
    }
}

impl Type for FinalizersEntryType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.0
    }
}
