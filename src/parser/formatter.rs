use crate::ast::*;

impl Expression {
  /// Converts the expression to a string.
  pub fn as_string(&self) -> String {
    match self {
      Expression::VariableRef(s) => format!("{}", s),
      Expression::String(s) => format!("\"{}\"", s),
      Expression::Char(s) => format!("'{}'", s),
      Expression::Comment(s) if s.contains("\n") => format!("/* {} */", s),
      Expression::Comment(s) => format!("// {}", s),
      Expression::Bool(b) => format!("{}", b),
      Expression::Declare(b) => format!("declare {}", b.as_string()),
      Expression::FuncCall(name, args) => format!(
        "{}({})",
        name,
        args
          .iter()
          .map(|x| x.as_string())
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Expression::Number(s) | Expression::Decimal(s) => {
        let mut pre_decimal = String::new();
        let decimal_parts: Vec<&str> = s.split(".").collect();
        let enumerated = decimal_parts[0].chars().rev().enumerate();
        for (idx, val) in enumerated {
          if idx != 0 && idx % 3 == 0 {
            pre_decimal.insert(0, '_');
          }
          pre_decimal.insert(0, val);
        }
        if decimal_parts.len() == 1 {
          pre_decimal
        } else {
          format!("{}.{}", pre_decimal, decimal_parts[1])
        }
      }
      Expression::Block { expressions } => {
        format!(
          "{{{}}}",
          expressions
            .iter()
            .map(|x| x.as_string())
            .collect::<Vec<String>>()
            .join("\n")
        )
      }
      Expression::Function {
        name,
        ret,
        args,
        body,
      } => {
        let body_str = if let Some(bod) = body {
          format!(" {{{}}}", bod.as_string())
        } else {
          format!(";")
        };
        format!(
          "function {}({}) -> {}{}",
          name,
          args
            .iter()
            .map(|(arg_name, arg_type)| format!("{}: {}", arg_name, arg_type))
            .collect::<Vec<String>>()
            .join(","),
          ret,
          body_str
        )
      }
      Expression::Return(expr) => {
        if let Some(ret) = expr {
          format!("return {};", ret.as_string())
        } else {
          "return;".to_owned()
        }
      }
      Expression::VariableDeclaration { name, ty, mutable } => {
        format!(
          "{} {}: {}",
          if *mutable { "let" } else { "const" },
          name,
          ty
        )
      }

      // Loops
      Expression::For { conditions, body } => {
        format!(
          "for ({}; {}; {}) {}",
          conditions[0].as_string(),
          conditions[1].as_string(),
          conditions[2].as_string(),
          body.as_string()
        )
      }
      Expression::While { condition, body } => {
        format!("while ({}) {}", condition.as_string(), body.as_string())
      }

      // Control flow
      Expression::If { condition, body } => {
        format!("if ({}) {}", condition.as_string(), body.as_string())
      }
      Expression::Else { body } => {
        format!("else {}", body.as_string())
      }

      // Module Logic
      Expression::Import { path, idents, .. } => {
        let import_expr = if let Some(idents) = idents {
          format!("{{{}}}", idents.join(", "))
        } else {
          format!("*")
        };
        format!("import {} from \"{}\"", import_expr, path)
      }
      Expression::Export(expr) => format!("export {};", expr.as_string()),
      Expression::ExportFromFile { path, idents, .. } => {
        let import_expr = if let Some(idents) = idents {
          format!("{{{}}}", idents.join(", "))
        } else {
          format!("*")
        };
        format!("import {} from \"{}\"", import_expr, path)
      }

      // Operations
      Expression::UnaryOperation {
        operator,
        expr,
        position,
      } => {
        let op_str = operator.as_string();
        let expr_str = expr.as_string();
        match position {
          OperatorPosition::Prefix => format!("{}{}", op_str, expr_str),
          OperatorPosition::Postfix => format!("{}{}", expr_str, op_str),
        }
      }
      Expression::BinaryOperation { operator, lhs, rhs } => {
        let op_str = operator.as_string();
        format!("{} {} {}", lhs.as_string(), op_str, rhs.as_string())
      }
    }
  }
}

impl UnaryOperator {
  /// Gets the string representation of the operator.
  pub fn as_string(&self) -> String {
    String::from(match self {
      UnaryOperator::Ref => "&",
      UnaryOperator::Deref => "*",
      UnaryOperator::Not => "!",
      UnaryOperator::Increment => "++",
      UnaryOperator::Decrement => "--",
      UnaryOperator::BitNOT => "~",
      UnaryOperator::Negation => "-",
    })
  }
}

impl BinaryOperator {
  /// Gets the string representation of the operator.
  pub fn as_string(&self) -> String {
    String::from(match self {
      BinaryOperator::Assign => "=",
      BinaryOperator::Add => "+",
      BinaryOperator::Subtract => "-",
      BinaryOperator::Multiply => "*",
      BinaryOperator::Divide => "/",
      BinaryOperator::Modulo => "%",
      BinaryOperator::AddAssign => "+=",
      BinaryOperator::SubtAssign => "-=",
      BinaryOperator::MultAssign => "*=",
      BinaryOperator::DivAssign => "/=",
      BinaryOperator::ModAssign => "%=",
      BinaryOperator::LeftShiftAssign => "<<=",
      BinaryOperator::RightShiftAssign => ">>=",
      BinaryOperator::BitANDAssign => "&=",
      BinaryOperator::BitXORAssign => "^=",
      BinaryOperator::BitORAssign => "|=",
      BinaryOperator::Eq => "==",
      BinaryOperator::Ne => "!=",
      BinaryOperator::Gt => ">",
      BinaryOperator::Lt => "<",
      BinaryOperator::Ge => ">=",
      BinaryOperator::Le => "<=",
      BinaryOperator::BitAND => "&",
      BinaryOperator::BitXOR => "^",
      BinaryOperator::BitOR => "||",
      BinaryOperator::LeftShift => "<<",
      BinaryOperator::RightShift => ">>",
    })
  }
}
