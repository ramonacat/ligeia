use std::{ffi::CString, marker::PhantomData, str::FromStr};

use llvm_sys::{
    core::{
        LLVMBuildStructGEP2, LLVMConstNamedStruct, LLVMCountStructElementTypes,
        LLVMStructCreateNamed, LLVMStructSetBody,
    },
    prelude::LLVMTypeRef,
};

use super::{Type, value::DynamicValue};
use crate::{
    Context, LLVM_CONTEXT,
    function::instruction_builder::InstructionBuilder,
    types::value::{ConstValue, Value},
};

// TODO: A proc derive macro that generates cute structs that match on both the Rust side, and the
// FFI side
pub struct Struct {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl Struct {
    /// # Panics
    /// This function will panic if the name can't be converted into a `CString`
    pub fn new(name: &str, fields: &[&dyn Type]) -> Self {
        let name = CString::from_str(name).unwrap();
        let reference = LLVM_CONTEXT
            // SAFETY: The context is &'static so must always be valid, the name is a valid pointer
            // for the duration of the call
            .with(|context| unsafe { LLVMStructCreateNamed(context.as_llvm_ref(), name.as_ptr()) });
        let mut elements: Vec<_> = fields.iter().map(|x| x.as_llvm_ref()).collect();

        // SAFETY: The reference was just created, so it's valid, the elements vector is alive and
        // the length and type match expectations of the method called.
        unsafe {
            LLVMStructSetBody(
                reference,
                elements.as_mut_ptr(),
                u32::try_from(elements.len()).unwrap(),
                0,
            );
        };

        Self {
            reference,
            _context: PhantomData,
        }
    }

    /// # Panics
    /// This will panic if the number of field values does not match the number of defined fields.
    /// TODO Should we return an error in that case instead?
    #[must_use]
    pub fn const_value(&self, fields: &[ConstValue]) -> ConstValue {
        assert!(self.fields_count() == fields.len());

        let mut values: Vec<_> = fields.iter().map(Value::as_llvm_ref).collect();

        // SAFETY: the values vector is alive, rightly typed and the passed length matches, the
        // self.refernce must be valid until self is dropped
        let value = unsafe {
            LLVMConstNamedStruct(
                self.reference,
                values.as_mut_ptr(),
                u32::try_from(values.len()).unwrap(),
            )
        };

        // SAFETY: We just created the value so it's a valid one
        unsafe { ConstValue::new(value) }
    }

    /// # Panics
    /// If the field index is out of bounds
    #[must_use]
    pub fn get_field_pointer(
        &self,
        i: &InstructionBuilder,
        pointer: &DynamicValue,
        index: usize,
        name: &str,
    ) -> Option<DynamicValue> {
        if index >= self.fields_count() {
            return None;
        }

        let name = CString::new(name).unwrap();

        // SAFETY: We have a valid builder, valid type reference, a valid pointer, valid name and
        // the index has been bounds checked.
        let value = unsafe {
            LLVMBuildStructGEP2(
                i.builder(),
                self.reference,
                pointer.as_llvm_ref(),
                u32::try_from(index).unwrap(),
                name.as_ptr(),
            )
        };

        // SAFETY: We just created the value, so it is a valid pointer
        Some(unsafe { DynamicValue::new(value) })
    }

    fn fields_count(&self) -> usize {
        // SAFETY: We know that the struct reference is valid
        (unsafe { LLVMCountStructElementTypes(self.reference) }) as usize
    }
}

impl Type for Struct {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}
