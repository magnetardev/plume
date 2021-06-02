use super::target_machine::TargetMachine;
use llvm_sys::core::*;
use llvm_sys::target_machine::*;
use llvm_sys::LLVMModule;
use std::ffi::{CStr, CString};
use std::path::Path;

/// A struct that keeps ownership of all the strings we've passed to
/// the LLVM API until we destroy the `LLVMModule`.
pub struct Module {
  pub module: *mut LLVMModule,
  pub strings: Vec<CString>,
}

impl Module {
  /// Create a new CString associated with this LLVMModule,
  /// and return a pointer that can be passed to LLVM APIs.
  /// Assumes s is pure-ASCII.
  pub fn new_string_ptr(&mut self, s: &str) -> *const i8 {
    self.new_mut_string_ptr(s)
  }

  // TODO: ideally our pointers wouldn't be mutable.
  pub fn new_mut_string_ptr(&mut self, s: &str) -> *mut i8 {
    let cstring = CString::new(s).unwrap();
    let ptr = cstring.as_ptr() as *mut _;
    self.strings.push(cstring);
    ptr
  }

  pub fn write_ir_file(&mut self, path: &Path) -> Result<(), String> {
    unsafe {
      path.to_str().unwrap();
      let mut obj_error = self.new_mut_string_ptr("Writing LLVM IR file failed.");
      let result = LLVMPrintModuleToFile(
        self.module,
        self.new_string_ptr(path.to_str().unwrap()) as *mut i8,
        &mut obj_error,
      );
      if result != 0 {
        return Err(
          CStr::from_ptr(obj_error as *const _)
            .to_str()
            .unwrap()
            .to_string(),
        );
      }
    }
    Ok(())
  }

  pub fn write_object_file(&mut self, path: &Path) -> Result<(), String> {
    unsafe {
      let target_triple = LLVMGetTarget(self.module);
      let target_machine = TargetMachine::new(target_triple)?;

      let mut obj_error = self.new_mut_string_ptr("Writing object file failed.");
      let result = LLVMTargetMachineEmitToFile(
        target_machine.tm,
        self.module,
        self.new_string_ptr(path.to_str().unwrap()) as *mut i8,
        LLVMCodeGenFileType::LLVMObjectFile,
        &mut obj_error,
      );

      if result != 0 {
        return Err(
          CStr::from_ptr(obj_error as *const _)
            .to_str()
            .unwrap()
            .to_string(),
        );
      }
    }
    Ok(())
  }
}

impl Drop for Module {
  fn drop(&mut self) {
    // Rust requires that drop() is a safe function.
    unsafe {
      LLVMDisposeModule(self.module);
    }
  }
}
