use llvm_sys::core::*;
use llvm_sys::target_machine::*;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::str;

pub struct TargetMachine {
  pub tm: LLVMTargetMachineRef,
}

impl TargetMachine {
  pub fn new(target_triple: *const i8) -> Result<Self, String> {
    let mut target = null_mut();
    let mut err_msg_ptr = null_mut();
    unsafe {
      LLVMGetTargetFromTriple(target_triple, &mut target, &mut err_msg_ptr);
      if target.is_null() {
        // LLVM couldn't find a target triple with this name,
        // so it should have given us an error message.
        assert!(!err_msg_ptr.is_null());

        let err_msg_cstr = CStr::from_ptr(err_msg_ptr as *const _);
        let err_msg = str::from_utf8(err_msg_cstr.to_bytes()).unwrap();
        return Err(err_msg.to_owned());
      }
    }

    // TODO: do these strings live long enough?
    // cpu is documented: http://llvm.org/docs/CommandGuide/llc.html#cmdoption-mcpu
    let cpu = CString::new("generic").unwrap();
    // features are documented: http://llvm.org/docs/CommandGuide/llc.html#cmdoption-mattr
    let features = CString::new("").unwrap();

    let target_machine;
    unsafe {
      target_machine = LLVMCreateTargetMachine(
        target,
        target_triple,
        cpu.as_ptr() as *const _,
        features.as_ptr() as *const _,
        LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        LLVMRelocMode::LLVMRelocPIC,
        LLVMCodeModel::LLVMCodeModelDefault,
      );
    }

    Ok(TargetMachine { tm: target_machine })
  }

  pub fn get_default_target_triple() -> CString {
    let target_triple;
    unsafe {
      let target_triple_ptr = LLVMGetDefaultTargetTriple();
      target_triple = CStr::from_ptr(target_triple_ptr as *const _).to_owned();
      LLVMDisposeMessage(target_triple_ptr);
    }
    target_triple
  }
}

impl Drop for TargetMachine {
  fn drop(&mut self) {
    unsafe {
      LLVMDisposeTargetMachine(self.tm);
    }
  }
}
