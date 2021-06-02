#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
  Number(String),
  Decimal(String),
  String(String),
  Char(char),
  Bool(bool),
  VariableRef(String),
  Comment(String),
  Return(Option<Box<Expression>>),
  FuncCall(String, Vec<Expression>),
  Export(Box<Expression>),
  Declare(Box<Expression>),
  Import {
    idents: Option<Vec<String>>,
    import_all: bool,
    path: String,
  },
  ExportFromFile {
    idents: Option<Vec<String>>,
    export_all: bool,
    path: String,
  },
  Function {
    name: String,
    ret: String,
    args: Vec<(String, String)>,
    body: Option<Box<Expression>>,
  },
  Block {
    expressions: Vec<Expression>,
  },
  VariableDeclaration {
    name: String,
    ty: String,
    mutable: bool,
  },
  For {
    conditions: [Box<Expression>; 3],
    body: Box<Expression>,
  },
  While {
    condition: Box<Expression>,
    body: Box<Expression>,
  },
  If {
    condition: Box<Expression>,
    body: Box<Expression>,
  },
  Else {
    body: Box<Expression>,
  },
  UnaryOperation {
    operator: UnaryOperator,
    expr: Box<Expression>,
    position: OperatorPosition,
  },
  BinaryOperation {
    operator: BinaryOperator,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
  },
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
  Not,       // !
  Deref,     // *
  Ref,       // &
  Increment, // ++
  Decrement, // --
  BitNOT,    // ~
  Negation,  // -
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorPosition {
  Prefix,
  Postfix,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
  Assign,   // =
  Add,      // +
  Subtract, // -
  Multiply, // *
  Divide,   // /
  Modulo,   // %

  AddAssign,        // +=
  SubtAssign,       // -=
  MultAssign,       // *=
  DivAssign,        // /=
  ModAssign,        // %=
  LeftShiftAssign,  // <<=
  RightShiftAssign, // >>=
  BitANDAssign,     // &=
  BitXORAssign,     // ^=
  BitORAssign,      // |=

  // Comparisons
  Eq, // ==
  Ne, // !=
  Gt, // >
  Lt, // <
  Ge, // >=
  Le, // <=

  // Bitwise
  BitAND,     // &
  BitXOR,     // ^
  BitOR,      // ||
  LeftShift,  // <<
  RightShift, // >>
}

impl UnaryOperator {
  /// Construct a Unary Operator from its string literal.
  pub fn from<S: AsRef<str>>(literal: S) -> Option<Self> {
    match literal.as_ref() {
      "!" => Some(Self::Not),
      "&" => Some(Self::Ref),
      "*" => Some(Self::Deref),
      "++" => Some(Self::Increment),
      "--" => Some(Self::Decrement),
      "~" => Some(Self::BitNOT),
      "-" => Some(Self::Negation),
      _ => None,
    }
  }
}

impl BinaryOperator {
  /// Construct a Binary Operator from its string literal.
  pub fn from<S: AsRef<str>>(literal: S) -> Option<Self> {
    match literal.as_ref() {
      "=" => Some(Self::Assign),   // =
      "+" => Some(Self::Add),      // +
      "-" => Some(Self::Subtract), // -
      "*" => Some(Self::Multiply), // *
      "/" => Some(Self::Divide),   // /
      "%" => Some(Self::Modulo),   // %

      "+=" => Some(Self::AddAssign),         // +=
      "-=" => Some(Self::SubtAssign),        // -=
      "*=" => Some(Self::MultAssign),        // *=
      "/=" => Some(Self::DivAssign),         // /=
      "%=" => Some(Self::ModAssign),         // %=
      "<<=" => Some(Self::LeftShiftAssign),  // <<=
      ">>=" => Some(Self::RightShiftAssign), // >>=
      "&=" => Some(Self::BitANDAssign),      // &=
      "^=" => Some(Self::BitXORAssign),      // ^=
      "|=" => Some(Self::BitORAssign),       // |=

      // Comparisons
      "==" => Some(Self::Eq), // ==
      "!=" => Some(Self::Ne), // !=
      ">" => Some(Self::Gt),  // >
      "<" => Some(Self::Lt),  // <
      ">=" => Some(Self::Ge), // >=
      "<=" => Some(Self::Le), // <=

      // Bitwise
      "&" => Some(Self::BitAND),      // &
      "^" => Some(Self::BitXOR),      // ^
      "||" => Some(Self::BitOR),      // ||
      "<<" => Some(Self::LeftShift),  // <<
      ">>" => Some(Self::RightShift), // >>
      _ => None,
    }
  }

  // /// The binding power of a operator.
  // /// This is used to determine the order of operations for multiple binary operations.
  // /// e.g. `1 + 1 * 2 => 1 + (1 * 2)`
  // pub fn bp(&self) -> u32 {
  //   match self {
  //     _ => 0,
  //   }
  // }
}
