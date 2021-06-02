use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMBuilder;

/// Wraps LLVM's builder class to provide a nicer API and ensure we
/// always dispose correctly.
pub struct Builder {
  pub builder: *mut LLVMBuilder,
}

impl Builder {
  /// Create a new Builder in LLVM's global context.
  pub fn new() -> Self {
    unsafe {
      Builder {
        builder: LLVMCreateBuilder(),
      }
    }
  }

  pub fn position_at_end(&self, bb: LLVMBasicBlockRef) {
    unsafe {
      LLVMPositionBuilderAtEnd(self.builder, bb);
    }
  }
}

impl Drop for Builder {
  fn drop(&mut self) {
    // Rust requires that drop() is a safe function.
    unsafe {
      LLVMDisposeBuilder(self.builder);
    }
  }
}
