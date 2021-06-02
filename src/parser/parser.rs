use crate::ast::{BinaryOperator, Expression, OperatorPosition, UnaryOperator};
use crate::lexer::{Lexer, Token, TokenKind};
pub struct Parser {
  lexer: Lexer,
}

pub type Expressions = Vec<Expression>;

impl Parser {
  pub fn new(lexer: Lexer) -> Self {
    Self { lexer }
  }

  pub fn from<S: AsRef<str>>(source: S) -> Self {
    Self::new(Lexer::new(source.as_ref().to_string()))
  }

  fn peek_is_kind(&mut self, kind: TokenKind) -> bool {
    self.lexer.peek().filter(|x| x.is_kind(kind)).is_some()
  }

  // fn peek_is_lit<S: AsRef<str>>(&mut self, literal: S) -> bool {
  //   self.lexer.peek().filter(|x| x.is_lit(literal)).is_some()
  // }

  fn peek_cmp_token<S: AsRef<str>>(&mut self, kind: TokenKind, literal: S) -> bool {
    self
      .lexer
      .peek()
      .filter(|x| x.cmp_token(kind, literal))
      .is_some()
  }

  pub fn parse(&mut self) -> Expressions {
    let mut program = Expressions::new();
    while let Some(token) = self.lexer.next() {
      if let Some(expr) = self.parse_expression(Some(token), false) {
        program.push(expr);
      }
    }
    return program;
  }

  fn parse_expression(&mut self, token: Option<Token>, ignore_op: bool) -> Option<Expression> {
    if let Some(tok) = token {
      match tok.kind {
        TokenKind::UnaryOperator | TokenKind::SomeOperator => {
          if let Some(operator) = UnaryOperator::from(tok.literal) {
            let next_token = self.lexer.next();
            let expr = self
              .parse_expression(next_token, false)
              .map(|x| Box::new(x))
              .expect("Expected expression");
            Some(Expression::UnaryOperation {
              operator,
              expr,
              position: OperatorPosition::Prefix,
            })
          } else {
            None
          }
        }
        _ if ignore_op == false && self.peek_is_kind(TokenKind::UnaryOperator) => {
          let expr = Box::new(
            self
              .parse_expression(Some(tok), true)
              .expect("Expected lhs in binary operation"),
          );
          let op = self
            .lexer
            .next()
            .map(|x| UnaryOperator::from(x.literal))
            .expect("Expected an operator")
            .expect("Invalid operator");
          Some(Expression::UnaryOperation {
            operator: op,
            expr,
            position: OperatorPosition::Postfix,
          })
        }
        _ if ignore_op == false
          && (self.peek_is_kind(TokenKind::BinaryOperator)
            || self.peek_is_kind(TokenKind::SomeOperator)) =>
        {
          let lhs = Box::new(
            self
              .parse_expression(Some(tok), true)
              .expect("Expected lhs in binary operation"),
          );
          let op = self
            .lexer
            .next()
            .map(|x| BinaryOperator::from(x.literal))
            .expect("Expected an operator")
            .expect("Invalid operator");
          let next_token = self.lexer.next();
          let rhs = Box::new(
            self
              .parse_expression(next_token, false)
              .expect("Expected rhs in binary operation"),
          );
          Some(Expression::BinaryOperation {
            operator: op,
            lhs,
            rhs,
          })
        }
        TokenKind::Keyword if tok.is_lit("function") => Some(self.parse_function()),
        TokenKind::Keyword if tok.is_lit("import") => Some(self.parse_module_reference(true)),
        TokenKind::Keyword
          if tok.is_lit("export")
            && (self.peek_cmp_token(TokenKind::Braces, "{")
              || self.peek_cmp_token(TokenKind::SomeOperator, "*")) =>
        {
          Some(self.parse_module_reference(false))
        }
        TokenKind::Bool => Some(Expression::Bool(tok.is_lit("true"))),
        TokenKind::Keyword if tok.is_lit("declare") => {
          let next = self.lexer.next();
          Some(Expression::Declare(
            self
              .parse_expression(next, false)
              .map(|x| Box::new(x))
              .expect("Invalid declare syntax"),
          ))
        }
        TokenKind::Keyword if tok.is_lit("export") => {
          let next = self.lexer.next();
          Some(Expression::Export(
            self
              .parse_expression(next, false)
              .map(|x| Box::new(x))
              .expect("Invalid export syntax"),
          ))
        }
        TokenKind::Keyword if tok.is_lit("return") => {
          let next = self.lexer.next();
          Some(Expression::Return(
            self.parse_expression(next, false).map(|x| Box::new(x)),
          ))
        }
        TokenKind::Keyword if tok.is_lit("for") => Some(self.parse_for_loop()),
        TokenKind::Keyword if tok.is_lit("while") => Some(self.parse_control_flow("while")),
        TokenKind::Keyword if tok.is_lit("if") => Some(self.parse_control_flow("if")),
        TokenKind::Keyword if tok.is_lit("else") => {
          let next = self.lexer.next();
          Some(Expression::Else {
            body: self
              .parse_expression(next, false)
              .map(|x| Box::new(x))
              .expect("else statements require a body"),
          })
        }
        TokenKind::Keyword if tok.is_lit("let") || tok.is_lit("const") => {
          let mutable = tok.is_lit("let");
          let name = self
            .lexer
            .next()
            .filter(|x| x.is_kind(TokenKind::Ident))
            .map(|x| x.literal)
            .expect("");
          if !self.peek_is_kind(TokenKind::Colon) {
            panic!("Typing is required for variable declarations");
          }
          self.lexer.next();
          let variable_dec = Expression::VariableDeclaration {
            name,
            ty: self.parse_type(),
            mutable,
          };
          if self.peek_cmp_token(TokenKind::BinaryOperator, "=") {
            self.lexer.next();
            let next_token = self.lexer.next();
            let rhs = self
              .parse_expression(next_token, false)
              .map(|x| Box::new(x))
              .expect("Expected something to be assigned to the variable");
            Some(Expression::BinaryOperation {
              operator: BinaryOperator::Assign,
              lhs: Box::new(variable_dec),
              rhs,
            })
          } else {
            Some(variable_dec)
          }
        }
        // Function Call
        TokenKind::Ident if self.peek_cmp_token(TokenKind::Parens, "(") => {
          let ident = tok.literal;
          self
            .lexer
            .next()
            .filter(|x| x.is_lit("("))
            .expect("Invalid function call syntax");
          let mut args: Vec<Expression> = vec![];
          while let Some(t) = self.lexer.next() {
            if t.cmp_token(TokenKind::Parens, ")") {
              break;
            }
            if t.is_kind(TokenKind::Comma) {
              continue;
            }
            if let Some(expr) = self.parse_expression(Some(t), false) {
              args.push(expr);
            }
          }
          Some(Expression::FuncCall(ident, args))
        }
        // Variable reference
        TokenKind::Ident => {
          let ident = tok.literal;
          Some(Expression::VariableRef(ident))
        }
        // Block
        TokenKind::Braces if tok.is_lit("{") => Some(self.parse_block()),
        // String literal
        TokenKind::String => Some(Expression::String(tok.literal)),
        // Char literal
        TokenKind::Char => {
          let mut chars = tok.literal.chars();
          let first_char = chars
            .nth(0)
            .expect("SyntaxError: char literal contains no char.");
          if chars.count() == 0 && first_char.is_ascii() {
            Some(Expression::Char(first_char))
          } else {
            panic!("SyntaxError: char literal is larger than one byte.");
          }
        }
        // Decimal
        TokenKind::Number if tok.literal.contains(".") => Some(Expression::Decimal(tok.literal)),
        // Literal
        TokenKind::Number => Some(Expression::Number(tok.literal)),
        // Comment
        TokenKind::Comment => Some(Expression::Comment(tok.literal)),
        // Semicolon
        TokenKind::Semicolon => None,
        _ => None,
      }
    } else {
      None
    }
  }

  fn parse_function(&mut self) -> Expression {
    if !self.peek_is_kind(TokenKind::Ident) {
      panic!(
        "SyntaxError: Invalid function signature. Expected an identifier, found {:?}",
        self.lexer.peek()
      );
    }
    let ident_tok = self.lexer.next().unwrap();
    let open_parens = self.lexer.next().expect("msg: &str");
    // Parse args
    if !open_parens.cmp_token(TokenKind::Parens, "(") {
      panic!(
        "SyntaxError: Invalid function signature. Expected open parentheses, found {:?}",
        open_parens
      );
    }
    let mut args = Vec::<(String, String)>::new();
    while let Some(token) = self.lexer.next() {
      if token.cmp_token(TokenKind::Parens, ")") {
        break;
      }
      if token.is_kind(TokenKind::Ident) {
        let arg_name = token.literal;
        self
          .lexer
          .next()
          .filter(|x| x.is_kind(TokenKind::Colon))
          .expect("Expected type signature");
        let ty = self.parse_type();
        args.push((arg_name, ty));
      } else if !token.is_kind(TokenKind::Comma) {
        panic!("No type identifier.");
      }
    }

    // Parse return type
    let ret_type: String = if self.peek_is_kind(TokenKind::ReturnArrow) {
      self.lexer.next();
      if let Some(ret) = self.lexer.next().filter(|x| x.is_kind(TokenKind::Ident)) {
        ret.literal
      } else {
        panic!("SyntaxError: Invalid function signature, no return type specified after arrow.");
      }
    } else {
      self.lexer.next();
      "void".to_owned()
    };

    // Parse body
    let body: Option<Box<Expression>> = if self.peek_cmp_token(TokenKind::Braces, "{") {
      let next = self.lexer.next();
      self.parse_expression(next, false).map(|x| Box::new(x))
    } else {
      None
    };
    Expression::Function {
      name: ident_tok.literal,
      ret: ret_type,
      args: args,
      body: body,
    }
  }

  fn parse_block(&mut self) -> Expression {
    let mut expressions = Vec::<Expression>::new();
    while let Some(token) = self.lexer.next() {
      if token.cmp_token(TokenKind::Braces, "}") {
        break;
      }
      if let Some(expr) = self.parse_expression(Some(token), false) {
        expressions.push(expr);
      }
    }
    Expression::Block { expressions }
  }

  fn parse_control_flow<S: AsRef<str>>(&mut self, literal: S) -> Expression {
    self
      .lexer
      .next()
      .filter(|x| x.cmp_token(TokenKind::Parens, "("))
      .expect("Expected parens to start the condition");
    let condition_token = self.lexer.next();
    let condition = Box::new(
      self
        .parse_expression(condition_token, false)
        .expect("Expected a condition"),
    );
    self
      .lexer
      .next()
      .filter(|x| x.cmp_token(TokenKind::Parens, ")"))
      .expect("Expected parens to end the condition");
    let body_token = self.lexer.next();
    let body = self
      .parse_expression(body_token, false)
      .map(|x| Box::new(x))
      .expect("Expected a body");
    match literal.as_ref() {
      "if" => Expression::If { condition, body },
      "while" => Expression::While { condition, body },
      _ => unimplemented!(),
    }
  }

  fn parse_for_loop(&mut self) -> Expression {
    self
      .lexer
      .next()
      .filter(|x| x.cmp_token(TokenKind::Parens, "("))
      .expect("Expected parens to start a condition"); // (
    let cond_a_token = self.lexer.next();
    let cond_a = self
      .parse_expression(cond_a_token, false)
      .map(|x| Box::new(x))
      .expect("expected a condition");
    self
      .lexer
      .next()
      .filter(|x| x.is_kind(TokenKind::Semicolon))
      .expect("Expected semicolon"); // ;
    let cond_b_token = self.lexer.next();
    let cond_b = self
      .parse_expression(cond_b_token, false)
      .map(|x| Box::new(x))
      .expect("expected a condition");
    self
      .lexer
      .next()
      .filter(|x| x.is_kind(TokenKind::Semicolon))
      .expect("Expected semicolon"); // ;
    let cond_c_token = self.lexer.next();
    let cond_c = self
      .parse_expression(cond_c_token, false)
      .map(|x| Box::new(x))
      .expect("expected a condition");
    self
      .lexer
      .next()
      .filter(|x| x.cmp_token(TokenKind::Parens, ")"))
      .expect("Expected parens to end a condition"); // )
    let body_token = self.lexer.next();
    let body: Box<Expression> = self
      .parse_expression(body_token, false)
      .map(|x| Box::new(x))
      .expect("Expected a body for the for loop");
    Expression::For {
      conditions: [cond_a, cond_b, cond_c],
      body,
    }
  }

  /// Handles the parsing of an import or `export _ from "..."`
  fn parse_module_reference(&mut self, is_import: bool) -> Expression {
    let mut import_all = false;
    let mut idents: Option<Vec<String>> = None;
    // Parse imports
    if self.peek_cmp_token(TokenKind::Braces, "{") {
      let mut ident_vec = vec![];
      self.lexer.next();
      while let Some(token) = self.lexer.next() {
        if token.cmp_token(TokenKind::Braces, "}") {
          break;
        }
        if token.is_kind(TokenKind::Ident) {
          ident_vec.push(token.literal);
        }
      }
      idents = Some(ident_vec);
    } else if self.peek_cmp_token(TokenKind::SomeOperator, "*") {
      self.lexer.next();
      import_all = true;
    }
    // Expect "from" keyword
    if self
      .lexer
      .next()
      .filter(|x| x.cmp_token(TokenKind::Keyword, "from"))
      .is_none()
    {
      panic!("SyntaxError: Expected 'from' keyword in import.");
    }

    // Get path
    let path = self
      .lexer
      .next()
      .filter(|x| x.is_kind(TokenKind::String))
      .map(|x| x.literal)
      .expect("SyntaxError: Expected path in import.");

    // Return
    if is_import {
      Expression::Import {
        path,
        idents,
        import_all,
      }
    } else {
      Expression::ExportFromFile {
        path,
        idents,
        export_all: import_all,
      }
    }
  }

  fn parse_type(&mut self) -> String {
    let mut is_pointer: bool = false;
    let mut is_array: bool = false;
    let base_type = self
      .lexer
      .next()
      .filter(|x| x.is_kind(TokenKind::Ident))
      .expect("Missing or improper type signature");
    let arr_len: Option<String> = None;
    if self.peek_cmp_token(TokenKind::SomeOperator, "*") {
      self.lexer.next();
      is_pointer = true;
    } else if self.peek_cmp_token(TokenKind::Brackets, "[") {
      self.lexer.next();
      let next_token = self
        .lexer
        .next()
        .filter(|x| x.cmp_token(TokenKind::Brackets, "]"))
        .expect("SyntaxError: Invalid array type signature");
      self.lexer.next();
      if next_token.is_kind(TokenKind::Brackets) {
        is_array = true;
      } else {
        if self.peek_cmp_token(TokenKind::Brackets, "]") {
          is_array = true;
        } else {
          panic!("SyntaxError: Invalid array type signature");
        }
      }
    }
    if is_pointer {
      format!("{}*", base_type.literal)
    } else if is_array && arr_len.is_some() {
      format!("{}[{}]", base_type.literal, arr_len.unwrap())
    } else if is_array {
      format!("{}[]", base_type.literal)
    } else {
      String::from(base_type.literal)
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn function() {
    assert_eq!(
      Parser::from("function hello() { \"hello!\"; }").parse(),
      vec![Expression::Function {
        name: "hello".to_owned(),
        ret: "void".to_owned(),
        args: vec![],
        body: Some(Box::new(Expression::Block {
          expressions: vec![Expression::String("hello!".to_owned())]
        }))
      }]
    )
  }

  #[test]
  fn import() {
    // Import { print }
    assert_eq!(
      Parser::from("import { print } from \"util.plume\"").parse(),
      vec![Expression::Import {
        path: "util.plume".to_owned(),
        idents: Some(vec!["print".to_owned()]),
        import_all: false,
      }]
    );

    // Import { print, hello }
    assert_eq!(
      Parser::from("import { print, hello } from \"util.plume\"").parse(),
      vec![Expression::Import {
        path: "util.plume".to_owned(),
        idents: Some(vec!["print".to_owned(), "hello".to_owned()]),
        import_all: false,
      }]
    );

    // Import *
    assert_eq!(
      Parser::from("import * from \"util.plume\"").parse(),
      vec![Expression::Import {
        path: "util.plume".to_owned(),
        idents: None,
        import_all: true,
      }]
    );
  }
}
