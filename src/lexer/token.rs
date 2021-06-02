#[derive(Debug, PartialEq)]
pub struct Token {
  pub kind: TokenKind,
  pub literal: String,
  pub line: u32,
  pub col: u32,
}

impl Token {
  pub fn new<S: AsRef<str>>(kind: TokenKind, literal: S, line: u32, col: u32) -> Self {
    let str_ref = literal.as_ref().to_string();
    Self {
      kind,
      literal: str_ref,
      line,
      col,
    }
  }

  /// Checks if the current token kind is equal to the kind passed as an argument.
  pub fn is_kind(&self, kind: TokenKind) -> bool {
    self.kind == kind
  }

  /// Checks if the current token literal is equal to the literal passed as an argument.
  pub fn is_lit<S: AsRef<str>>(&self, literal: S) -> bool {
    self.literal == literal.as_ref()
  }

  /// Checks if the current token literal and kind is equal to the kind and literal passed as arguments.
  pub fn cmp_token<S: AsRef<str>>(&self, kind: TokenKind, literal: S) -> bool {
    self.is_lit(literal) && self.is_kind(kind)
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
  Ident,          // An identifier
  SomeOperator,   // Unknown operator type.
  UnaryOperator,  // `operator expr` or `expr operator`
  BinaryOperator, // `expr operator expr`
  Parens,         // ( )
  Braces,         // { }
  Brackets,       // [ ]
  Comma,          // ,
  Colon,          // :
  Semicolon,      // ;
  Keyword,        // Some keyword
  Comment,        // A comment
  String,         // A string literal
  Char,           // A character
  Number,         // Some number, with or without decimal, float/double/int determined later
  Bool,           // true false
  ReturnArrow,    // ->
  Whitespace,     // Self explanatory
}
