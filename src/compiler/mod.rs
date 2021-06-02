mod llvm;
use crate::{Program, SourceFile};

impl Program {
  pub fn compile(&self, target_triple: Option<String>) {
    llvm::init_llvm();
    for (path, file) in &self.files {
      println!("[plume] compiling \"{}\"", path);
      match file.compile(target_triple.clone()) {
        Ok(_) => println!("[plume] compiled \"{}\"", path),
        Err(err) => panic!("[plume] failed to compile \"{}\": {}", path, err),
      }
    }
  }
}

impl SourceFile {
  pub fn compile(&self, target_triple: Option<String>) -> Result<(), String> {
    llvm::compile(self, target_triple)
  }
}
