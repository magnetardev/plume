#[cfg(test)]
use super::token::TokenKind::*;
use super::*;
// use std::assert;
#[test]
fn peek() {
  let mut lexer = Lexer::new("function print_two(a: string, b: char*) -> char*;".to_owned());
  assert_eq!(lexer.peek(), lexer.next()); // keyword 'function'
  assert_eq!(lexer.peek(), lexer.next()); // ident 'print_two'
  assert_eq!(lexer.peek(), lexer.next()); // parens '('
  assert_eq!(lexer.peek(), lexer.next()); // ident 'a'
  assert_eq!(lexer.peek(), lexer.next()); // colon ':'
  assert_eq!(lexer.peek(), lexer.next()); // ident 'string'
  assert_eq!(lexer.peek(), lexer.next()); // comma ','
  assert_eq!(lexer.peek(), lexer.next()); // ident 'b'
  assert_eq!(lexer.peek(), lexer.next()); // colon ':'
  assert_eq!(lexer.peek(), lexer.next()); // ident 'char'
  assert_eq!(lexer.peek(), lexer.next()); // operator '*'
  assert_eq!(lexer.peek(), lexer.next()); // parens ')'
  assert_eq!(lexer.peek(), lexer.next()); // return arrow '->'
  assert_eq!(lexer.peek(), lexer.next()); // ident 'char'
  assert_eq!(lexer.peek(), lexer.next()); // operator '*'
  assert_eq!(lexer.peek(), lexer.next()); // operator 'semicolon'
  assert_eq!(lexer.peek(), None); // None
  assert_eq!(lexer.next(), None); // None
}

#[test]
fn operators() {
  let mut lexer = Lexer::new(
    "
    + ++ += 
    - -- -=
    * *=
    / /=
    % %=
    > >= >> >>=
    < <= << <<=
    == !=
    ! & ~
    "
    .to_owned(),
  );
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "+"));
  assert!(next_cmp_token(&mut lexer, UnaryOperator, "++"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "+="));
  assert!(next_cmp_token(&mut lexer, SomeOperator, "-"));
  assert!(next_cmp_token(&mut lexer, UnaryOperator, "--"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "-="));
  assert!(next_cmp_token(&mut lexer, SomeOperator, "*"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "*="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "/"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "/="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "%"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "%="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, ">"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, ">="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, ">>"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, ">>="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "<"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "<="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "<<"));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "<<="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "=="));
  assert!(next_cmp_token(&mut lexer, BinaryOperator, "!="));
  assert!(next_cmp_token(&mut lexer, UnaryOperator, "!"));
  assert!(next_cmp_token(&mut lexer, SomeOperator, "&"));
  assert!(next_cmp_token(&mut lexer, UnaryOperator, "~"));
}

#[allow(dead_code)]
fn next_cmp_token<S: AsRef<str>>(lexer: &mut Lexer, kind: TokenKind, lit: S) -> bool {
  lexer
    .next()
    .filter(|x| {
      println!("cmp {:?} to {:?} {}", x, kind, lit.as_ref());
      x.cmp_token(kind, lit)
    })
    .is_some()
}
