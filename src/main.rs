#![deny(clippy::all, clippy::pedantic, clippy::nursery, warnings)]

use std::{ffi::CStr, mem::MaybeUninit};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAdd, LLVMBuildRet, LLVMConstInt,
        LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDumpModule,
        LLVMFunctionType, LLVMGetParam, LLVMInt64TypeInContext, LLVMModuleCreateWithNameInContext,
        LLVMPositionBuilderAtEnd,
    },
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMGetFunctionAddress, LLVMLinkInMCJIT,
    },
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
};

fn main() {
    unsafe {
        let context = LLVMContextCreate();
        let u64_type = LLVMInt64TypeInContext(context);
        let module = LLVMModuleCreateWithNameInContext(c"main".as_ptr().cast(), context);
        let builder = LLVMCreateBuilderInContext(context);

        let mut param_types = [u64_type];
        let function_type = LLVMFunctionType(
            u64_type,
            param_types.as_mut_ptr(),
            u32::try_from(param_types.len()).unwrap(),
            0,
        );
        let function = LLVMAddFunction(module, c"main".as_ptr().cast(), function_type);

        let entry = LLVMAppendBasicBlock(function, c"entry".as_ptr().cast());
        LLVMPositionBuilderAtEnd(builder, entry);

        let value = LLVMConstInt(u64_type, 32, 0);
        let sum = LLVMBuildAdd(
            builder,
            value,
            LLVMGetParam(function, 0),
            c"sum".as_ptr().cast(),
        );
        LLVMBuildRet(builder, sum);

        LLVMDisposeBuilder(builder);

        let mut out_message = std::ptr::null_mut();
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMAbortProcessAction,
            &mut out_message,
        );

        LLVMDumpModule(module);

        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();

        let execution_engine = {
            let mut engine = MaybeUninit::uninit();
            let mut error = std::mem::zeroed();

            if LLVMCreateExecutionEngineForModule(engine.as_mut_ptr(), module, &mut error) != 0 {
                assert!(!error.is_null());
                panic!("{:?}", CStr::from_ptr(error));
            }

            engine.assume_init()
        };

        let main = LLVMGetFunctionAddress(execution_engine, c"main".as_ptr().cast());
        let callable_main: extern "C" fn(u64) -> u64 = std::mem::transmute(main);

        let result = callable_main(12);

        println!("::: {result}");
    }
}
