mod test;
mod token;
pub use token::{Token, TokenKind};

#[derive(Clone)]
pub struct Lexer {
  source: Vec<char>,
  current: usize,
  next: usize,
  char: char,
  line: u32,
  column: u32,
}

impl Lexer {
  /// Creates a new lexer from a string
  pub fn new(input: String) -> Self {
    let mut s = Self {
      source: input.chars().collect(),
      current: 0,
      next: 1,
      char: '\0',
      line: 1,
      column: 1,
    };
    s.char = s.source[s.current];
    s
  }

  /// Determines if a given string is a keyword
  fn is_keyword(string: &String) -> bool {
    match string.as_str() {
      "declare" | "import" | "export" | "from" | "function" | "return" | "let" | "const" | "if"
      | "else" | "match" | "for" | "while" | "as" => true,
      _ => false,
    }
  }

  /// Shifts the cursor over the source up by one, consuming a single char.
  fn read(&mut self) {
    if self.next >= self.source.len() {
      self.char = '\0'
    } else {
      self.char = self.source[self.next];
    }
    if self.char == '\n' {
      self.line += 1;
      self.column = 0;
    } else {
      self.column += 1
    };
    self.current = self.next;
    self.next = self.current + 1;
  }

  /// Iterates over all whitespace, as Plume does not care about whitespace.
  fn skip_whitespace(&mut self) {
    while self.char.is_whitespace() {
      self.read();
    }
  }

  /// Creates a token for the given kind and literal with position data.
  fn token_str<S: AsRef<str>>(&self, kind: TokenKind, string: S) -> Token {
    Token::new(kind, string, self.line, self.column)
  }

  /// Creates a token for the given kind and char with position data.
  fn token_char(&self, kind: TokenKind, ch: char) -> Token {
    self.token_str(kind, ch.to_string())
  }

  /// Finds the next available token.
  fn match_token(&mut self) -> Token {
    self.skip_whitespace();
    match self.char {
      '/'
        if self.source.get(self.next) == Some(&'/') || self.source.get(self.next) == Some(&'*') =>
      {
        let mut buffer = String::new();
        let is_multiline = self.source.get(self.next) == Some(&'*');
        self.read();
        self.read();
        while self.current < self.source.len() {
          if is_multiline && (self.char == '*' && self.source.get(self.next) == Some(&'/')) {
            self.read();
            self.read();
            break;
          }
          if !is_multiline && self.char == '\n' {
            break;
          }
          buffer.push(self.char);
          self.read();
        }
        self.token_str(TokenKind::Comment, buffer.trim())
      }
      '\'' | '"' => {
        let mut buffer = String::new();
        let is_char = self.char == '\'';
        self.read();
        while self.current < self.source.len() && self.char != '"' && self.char != '\'' {
          buffer.push(self.char);
          self.read();
        }
        self.read();
        self.token_str(
          if is_char {
            TokenKind::Char
          } else {
            TokenKind::String
          },
          buffer,
        )
      }
      _ if self.char.is_alphabetic() => {
        let mut buffer = String::new();
        buffer.push(self.char);
        self.read();
        while self.current < self.source.len() && self.char.is_alphanumeric() || self.char == '_' {
          buffer.push(self.char);
          self.read();
        }
        let kind = if Self::is_keyword(&buffer) {
          TokenKind::Keyword
        } else if buffer == "true" || buffer == "false" {
          TokenKind::Bool
        } else {
          TokenKind::Ident
        };
        self.token_str(kind, buffer)
      }
      _ if self.char.is_numeric() => {
        let mut buffer = String::new();
        buffer.push(self.char);
        self.read();
        let mut has_decimal = false;
        loop {
          if self.current >= self.source.len() {
            break;
          }
          // Skip over number separators.
          if self.char == '_' {
            self.read();
          }
          if !self.char.is_numeric() && self.char != '.' {
            break;
          }
          if has_decimal == true && self.char == '.' {
            break;
          }
          if self.char == '.' {
            has_decimal = true;
          }
          buffer.push(self.char);
          self.read();
        }
        self.token_str(TokenKind::Number, buffer)
      }
      '-' if self.source.get(self.next) == Some(&'>') => {
        self.read();
        self.read();
        self.token_str(TokenKind::ReturnArrow, "->".to_owned())
      }
      '~' => {
        let resp = self.token_char(TokenKind::UnaryOperator, self.char);
        self.read();
        resp
      }
      '&' => {
        let resp = self.token_char(TokenKind::SomeOperator, self.char);
        self.read();
        resp
      }
      ',' => {
        self.read();
        self.token_str(TokenKind::Comma, ",")
      }
      ':' => {
        self.read();
        self.token_str(TokenKind::Colon, ":")
      }
      ';' => {
        self.read();
        self.token_str(TokenKind::Semicolon, ";")
      }
      '(' | ')' => {
        let resp = self.token_char(TokenKind::Parens, self.char);
        self.read();
        resp
      }
      '[' | ']' => {
        let resp = self.token_char(TokenKind::Brackets, self.char);
        self.read();
        resp
      }
      '{' | '}' => {
        let resp = self.token_char(TokenKind::Braces, self.char);
        self.read();
        resp
      }
      _ => {
        let resp: Token = match self.char {
          // +=, -=, ++, --, +, and -
          '+' | '-' => {
            if self.peek_char() == Some(self.char) {
              self.read();
              self.token_str(
                TokenKind::UnaryOperator,
                format!("{}{}", self.char, self.char),
              )
            } else if self.peek_char() == Some('=') {
              let resp = self.token_str(TokenKind::BinaryOperator, format!("{}=", self.char));
              self.read();
              resp
            } else {
              self.token_char(
                if self.char == '-' {
                  TokenKind::SomeOperator
                } else {
                  TokenKind::BinaryOperator
                },
                self.char,
              )
            }
          }
          // *=, /=, %=, *, /, and %
          '*' | '/' | '%' => {
            if self.peek_char() == Some('=') {
              let resp = self.token_str(TokenKind::BinaryOperator, format!("{}=", self.char));
              self.read();
              resp
            } else {
              self.token_char(
                if self.char == '*' {
                  TokenKind::SomeOperator
                } else {
                  TokenKind::BinaryOperator
                },
                self.char,
              )
            }
          }
          // = and ==
          '=' => {
            if self.peek_char() == Some('=') {
              let resp = self.token_str(TokenKind::BinaryOperator, "==");
              self.read();
              resp
            } else {
              self.token_char(TokenKind::BinaryOperator, self.char)
            }
          }
          // ! and !=
          '!' => {
            if self.peek_char() == Some('=') {
              self.read();
              self.token_str(TokenKind::BinaryOperator, "!=")
            } else {
              self.token_char(TokenKind::UnaryOperator, self.char)
            }
          }
          // <<=, >>=, <<, >>, <, >, <=, and >=
          '>' | '<' => {
            if self.peek_char() == Some(self.char) {
              self.read();
              if self.peek_char() == Some('=') {
                let resp = self.token_str(
                  TokenKind::BinaryOperator,
                  format!("{}{}=", self.char, self.char),
                );
                self.read();
                resp
              } else {
                self.token_str(
                  TokenKind::BinaryOperator,
                  format!("{}{}", self.char, self.char),
                )
              }
            } else if self.peek_char() == Some('=') {
              let resp = self.token_str(TokenKind::BinaryOperator, format!("{}=", self.char));
              self.read();
              resp
            } else {
              self.token_char(TokenKind::BinaryOperator, self.char)
            }
          }
          _ => self.token_char(TokenKind::Whitespace, self.char),
        };
        self.read();
        resp
      }
    }
  }

  /// Returns the next token (if present) without modifying positioning, allowing you to peek at the next available token.
  pub fn peek(&mut self) -> Option<Token> {
    if self.next >= self.source.len() {
      return None;
    }

    let old_current = self.current;
    let old_next = self.next;
    let old_char = self.char;
    let old_line = self.line;
    let old_col = self.column;

    let token = self.match_token();

    self.current = old_current;
    self.next = old_next;
    self.char = old_char;
    self.line = old_line;
    self.column = old_col;
    Some(token)
  }

  fn peek_char(&mut self) -> Option<char> {
    if self.next >= self.source.len() {
      return None;
    }

    let old_current = self.current;
    let old_next = self.next;
    let old_char = self.char;
    let old_line = self.line;
    let old_col = self.column;
    self.current = old_current;
    self.read();
    let resp = self.char;
    self.next = old_next;
    self.char = old_char;
    self.line = old_line;
    self.column = old_col;
    Some(resp)
  }
}

impl Iterator for Lexer {
  type Item = Token;

  /// Iterate over available tokens.
  fn next(&mut self) -> Option<Token> {
    if self.next >= self.source.len() {
      return None;
    }

    let token = self.match_token();
    Some(token)
  }
}
