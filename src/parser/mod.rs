mod formatter;
mod parser;
mod validator;
use crate::ast::Expression;
use parser::{Expressions, Parser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SourceFile {
  pub path: String,
  // hash: String,
  pub dependencies: Vec<String>,
  pub expressions: Expressions,
}

impl SourceFile {
  pub fn new<S: AsRef<str>>(path: S) -> SourceFile {
    println!("[plume] parsing \"{}\" ", path.as_ref());
    match fs::read_to_string(Path::new(path.as_ref())) {
      Ok(source) => {
        let mut parser = Parser::from(source);
        let expressions = parser.parse();
        let dependencies = Self::get_depends(&expressions);
        SourceFile {
          path: path.as_ref().to_string(),
          dependencies,
          expressions,
        }
      }
      Err(err) => panic!(
        "An error occurred while trying to open file at path '{}': {}",
        path.as_ref(),
        err
      ),
    }
  }

  /// Gets the dependencies of a file
  fn get_depends(expressions: &Vec<Expression>) -> Vec<String> {
    let dependencies: Vec<String> = expressions
      .iter()
      .filter(|x| match x {
        Expression::Import { .. } | Expression::ExportFromFile { .. } => true,
        _ => false,
      })
      .map(|x| match x {
        Expression::Import { path, .. } | Expression::ExportFromFile { path, .. } => path.clone(),
        _ => unreachable!(),
      })
      .collect();
    dependencies
  }

  /// Formats the file
  pub fn format(&self) -> String {
    self
      .expressions
      .iter()
      .map(|x| x.as_string())
      .collect::<Vec<String>>()
      .join("\n")
  }

  /// Validates that the file has proper syntax and logic, on top of what was already done with the parser.
  pub fn validate(&self) -> bool {
    return true;
  }
}

#[derive(Debug)]
pub struct Program {
  pub files: HashMap<String, SourceFile>,
}

impl Program {
  pub fn new<S: AsRef<str>>(path: S) -> Program {
    let entry_file = SourceFile::new(path);
    let mut files: HashMap<String, SourceFile> = HashMap::new();
    Self::resolve_depends(Path::new(&entry_file.path), &mut files, &entry_file);
    files.insert(entry_file.path.clone(), entry_file);
    Program { files }
  }

  fn resolve_depends(path: &Path, files: &mut HashMap<String, SourceFile>, source: &SourceFile) {
    for depend in &source.dependencies {
      let depend_path = path.with_file_name(depend);
      let depend_path_str = depend_path.to_str().expect("error").to_string();
      if files.contains_key(&depend_path_str) {
        continue;
      };
      let file = SourceFile::new(depend_path_str.clone());
      files.insert(depend_path_str, file.clone());
      Self::resolve_depends(&depend_path, files, &file);
    }
  }

  pub fn validate(&self) {
    for (path, source) in &self.files {
      println!("VALIDATION \"{}\": {}", path, source.validate());
    }
  }
}
