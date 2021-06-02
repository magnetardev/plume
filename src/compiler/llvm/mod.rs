mod builder;
mod module;
mod target_machine;
use crate::ast::*;
use crate::parser::SourceFile;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::{LLVMAttributeIndex, LLVMIntPredicate, LLVMLinkage};

use builder::Builder;
use llvm_sys::target::*;
use module::Module;
use target_machine::TargetMachine;

use std::ffi::CString;
use std::os::raw::{c_char, c_uint, c_ulonglong};
use std::path::Path;
use std::ptr;

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

macro_rules! c_str {
  ($s:expr) => {
    concat!($s, "\0").as_ptr() as *const i8
  };
}

struct CompileContext {
  pub module: Module,
  pub context: LLVMContextRef,
  pub attribute_index: LLVMAttributeIndex,
  pub functions: Vec<FunctionRef>,
}

struct FunctionRef {
  pub fn_ref: LLVMValueRef,
  pub args: Vec<(String, LLVMTypeRef)>,
  pub name: String,
  pub exported: bool,
  pub external: bool,
  /// Vec<(name, mutable, value)>
  pub vars: Vec<(String, bool, LLVMValueRef)>,
}

pub fn compile(source: &SourceFile, target_triple: Option<String>) -> Result<(), String> {
  let c_module_name = CString::new(source.path.clone()).unwrap();
  let module_name_char_ptr = c_module_name.to_bytes_with_nul().as_ptr() as *const _;
  let llvm_module = unsafe { LLVMModuleCreateWithName(module_name_char_ptr) };
  let mut ctx = CompileContext {
    module: Module {
      module: llvm_module,
      strings: vec![c_module_name],
    },
    context: unsafe { LLVMContextCreate() },
    attribute_index: 0,
    functions: vec![],
  };

  let target_triple_cstring = if let Some(target_triple) = target_triple {
    CString::new(target_triple).unwrap()
  } else {
    TargetMachine::get_default_target_triple()
  };

  // This is necessary for maximum LLVM performance, see
  // http://llvm.org/docs/Frontend/PerformanceTips.html
  unsafe {
    LLVMSetTarget(llvm_module, target_triple_cstring.as_ptr() as *const _);
  }
  for expr in &source.expressions {
    unsafe {
      compile_expression(&mut ctx, expr, false, false);
    }
  }

  let path = Path::new(&source.path);
  match ctx
    .module
    .write_ir_file(path.with_extension("ll").as_path())
  {
    Ok(_) => println!("[plume] exported LLVM IR \"{}\"", source.path),
    Err(err) => println!("[plume] error while exporting LLVM IR: {}", err),
  };
  ctx
    .module
    .write_object_file(path.with_extension("o").as_path())
}

pub fn init_llvm() {
  unsafe {
    // TODO: are all these necessary? Are there docs?
    LLVM_InitializeAllTargetInfos();
    LLVM_InitializeAllTargets();
    LLVM_InitializeAllTargetMCs();
    LLVM_InitializeAllAsmParsers();
    LLVM_InitializeAllAsmPrinters();
  }
}

unsafe fn compile_expression(
  ctx: &mut CompileContext,
  expression: &Expression,
  ext: bool,
  exported: bool,
) {
  // println!("compile_expression: {:?}", expression);
  match expression {
    Expression::Export(expr) => {
      compile_expression(ctx, expr, true, true);
    }
    Expression::Declare(expr) => {
      compile_expression(ctx, expr, true, false);
    }
    Expression::Function {
      name,
      ret,
      args,
      body,
    } => {
      let typed_args: Vec<(String, LLVMTypeRef)> = args
        .iter()
        .map(|(name, ty)| (name.clone(), get_type(ty)))
        .collect();
      let mut args_type: Vec<LLVMTypeRef> = typed_args.iter().map(|(_, ty)| ty.clone()).collect();
      let fn_type = LLVMFunctionType(
        get_type(ret),
        args_type.as_mut_ptr(),
        args.len() as u32,
        LLVM_FALSE,
      );
      let func = LLVMAddFunction(
        ctx.module.module,
        ctx.module.new_string_ptr(name.as_ref()),
        fn_type,
      );
      if ext {
        LLVMSetLinkage(func, LLVMLinkage::LLVMExternalLinkage)
      }
      if exported {}
      if let Some(body) = body {
        let func_ref = FunctionRef {
          fn_ref: func,
          args: typed_args,
          name: name.to_string(),
          exported,
          external: ext,
          vars: vec![],
        };
        let bb = LLVMAppendBasicBlock(func, ctx.module.new_string_ptr("entry"));
        build_body(ctx, bb, Some(&func_ref), body);
        ctx.functions.push(func_ref);
      }
    }
    _ => {}
  }
}

unsafe fn build_body(
  ctx: &mut CompileContext,
  bb: LLVMBasicBlockRef,
  func_ref: Option<&FunctionRef>,
  expression: &Expression,
) -> Option<LLVMValueRef> {
  // println!("build body expr: {:?}", expression);
  match expression {
    // Expression::FuncCall(name, args) => {
    //   build_func_call(module, bb, name, args, "");
    // }
    Expression::Number(value) => Some(get_value(&String::from("i8"), value)),
    Expression::Char(value) => Some(get_value(&String::from("char"), &value.to_string())),
    Expression::FuncCall(name, args) => {
      Some(build_func_call(ctx, func_ref, bb, &name.as_str(), args, ""))
    }
    Expression::VariableRef(name) => {
      if let Some(fn_ref) = func_ref {
        if let Some(pos) = fn_ref.args.iter().position(|(x, _)| x == name) {
          return Some(LLVMGetParam(fn_ref.fn_ref, pos as c_uint));
        } else if let Some(var) = fn_ref.vars.iter().find(|(x, _, _)| x == name) {
          return Some(var.2);
        } else {
          let global = LLVMGetNamedGlobal(
            ctx.module.module,
            ctx.module.new_mut_string_ptr(name.as_str()),
          );
          if global == ptr::null_mut() {
            return None;
          } else {
            return Some(global);
          }
        }
      }
      None
    }
    // Expression::UnaryOperation { expr, operator, position } => {

    // },
    Expression::BinaryOperation { lhs, rhs, operator } => {
      let lhs = build_body(ctx, bb, func_ref, lhs).expect("");
      let rhs = build_body(ctx, bb, func_ref, rhs).expect("");
      let builder = Builder::new();
      builder.position_at_end(bb);
      match operator {
        BinaryOperator::Add => Some(LLVMBuildAdd(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::Subtract => Some(LLVMBuildSub(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::Multiply => Some(LLVMBuildMul(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::Divide => Some(LLVMBuildFDiv(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::BitAND => Some(LLVMBuildAnd(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::BitOR => Some(LLVMBuildOr(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::BitXOR => Some(LLVMBuildXor(builder.builder, lhs, rhs, c_str!(""))),
        BinaryOperator::Eq => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntEQ,
          lhs,
          rhs,
          c_str!(""),
        )),
        BinaryOperator::Ne => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntNE,
          lhs,
          rhs,
          c_str!(""),
        )),
        BinaryOperator::Ge => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntSGE,
          lhs,
          rhs,
          c_str!(""),
        )),
        BinaryOperator::Gt => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntSGE,
          lhs,
          rhs,
          c_str!(""),
        )),
        BinaryOperator::Le => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntSLT,
          lhs,
          rhs,
          c_str!(""),
        )),
        BinaryOperator::Lt => Some(LLVMBuildICmp(
          builder.builder,
          LLVMIntPredicate::LLVMIntSLE,
          lhs,
          rhs,
          c_str!(""),
        )),
        _ => None,
      }
    }
    Expression::Return(expr) => {
      let builder = Builder::new();
      builder.position_at_end(bb);
      if let Some(expr) = expr {
        if let Some(value) = build_body(ctx, bb, func_ref, expr) {
          Some(LLVMBuildRet(builder.builder, value))
        } else {
          None
        }
      } else {
        Some(LLVMBuildRetVoid(builder.builder))
      }
    }
    Expression::Block { expressions } => {
      for expr in expressions {
        build_body(ctx, bb, func_ref, expr);
      }
      None
    }
    _ => None,
  }
}

unsafe fn build_func_call(
  ctx: &mut CompileContext,
  func_ref: Option<&FunctionRef>,
  bb: LLVMBasicBlockRef,
  fn_name: &str,
  args: &Vec<Expression>,
  name: &str,
) -> LLVMValueRef {
  let builder = Builder::new();
  builder.position_at_end(bb);

  let function = LLVMGetNamedFunction(ctx.module.module, ctx.module.new_string_ptr(fn_name));
  let mut args_value: Vec<LLVMValueRef> = args
    .iter()
    .map(|value| build_body(ctx, bb, func_ref, value))
    .map(|x| x.unwrap())
    .collect();
  let resp = LLVMBuildCall(
    builder.builder,
    function,
    args_value.as_mut_ptr(),
    args_value.len() as c_uint,
    ctx.module.new_string_ptr(name),
  );
  return resp;
}

unsafe fn get_type(ty: &String) -> LLVMTypeRef {
  // println!("get_type: {}", ty);
  match ty.as_str() {
    _ if ty.ends_with("*") => {
      LLVMPointerType(get_type(&ty.strip_suffix("*").unwrap().to_string()), 0)
    }
    "u128" | "i128" => LLVMInt64Type(),
    "u64" | "i64" => LLVMInt64Type(),
    "u32" | "i32" => LLVMInt32Type(),
    "u16" | "i16" => LLVMInt16Type(),
    "u8" | "i8" | "char" => LLVMInt8Type(),
    "string" => get_type(&String::from("char*")),
    _ => LLVMVoidType(),
  }
}

unsafe fn get_value(ty: &String, value: &String) -> LLVMValueRef {
  // println!("get_value: {} {}", ty, value);
  match ty.as_str() {
    "u128" | "i128" | "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "u8" | "i8" => LLVMConstInt(
      get_type(ty),
      value.parse().unwrap(),
      llvm_bool(ty.starts_with("u")),
    ),
    "char" => {
      let char = value.chars().nth(0).unwrap() as c_char as c_ulonglong;
      LLVMConstInt(LLVMInt8Type(), char, LLVM_FALSE)
    }
    "string" => LLVMConstInt(get_type(ty), value.parse().unwrap(), LLVM_FALSE),
    _ => ptr::null_mut() as LLVMValueRef,
  }
}

fn llvm_bool(from: bool) -> i32 {
  if from {
    LLVM_TRUE
  } else {
    LLVM_FALSE
  }
}
