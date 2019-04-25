use std::fmt;

#[derive(Debug, Clone)]
pub enum Token {
  Identifier(String),
  Punctuation(char),
  Operator(String),

  String(String),
  Boolean(bool),
  Integer(i32),
  Float(f64),

  Array(Vec<CtToken>),
  Structure {
    name: String,
    args: Vec<CtToken>,
  },
  StructureCall {
    name: String,
    attribute: String,
    args: Option<Vec<CtToken>>,
  },

  Binary {
    operator: String,
    left: Box<CtToken>,
    right: Box<CtToken>,
  },
  Return(Box<CtToken>),
  Break,
  Continue,
  VariableCall(String),
  Variable {
    name: String,
    return_type: types::Type,
  },
  FunctionCall {
    name: String,
    args: Vec<CtToken>,
  },
  FunctionHeader(FunctionHeader),
  Function(Function),

  If(Vec<If>),
  For {
    body: Vec<CtToken>,
  },
  While {
    condition: Box<CtToken>,
    body: Vec<CtToken>,
  },

  Error(String),
}

pub type CtToken = Option<Token>;

#[derive(Debug, Clone)]
pub struct FunctionHeader {
  pub name: String,
  pub args: Vec<CtToken>,
  pub return_type: types::Type,
}

#[derive(Debug, Clone)]
pub struct Function {
  pub header: FunctionHeader,
  pub body: Vec<CtToken>,
}

#[derive(Debug, Clone)]
pub struct If {
  pub condition: Box<CtToken>,
  pub body: Vec<CtToken>,
  pub return_type: types::Type,
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Token::String(value) => write!(f, "{}", value),
      Token::Boolean(value) => write!(f, "{}", value),
      Token::Integer(value) => write!(f, "{}", value),
      Token::Float(value) => write!(f, "{}", value),
      _ => fmt::Debug::fmt(self, f),
    }
  }
}

pub fn get_type(token: &CtToken) -> String {
  match token {
    Some(t) => match t {
      Token::String(_) => String::from(types::STRING),
      Token::Boolean(_) => String::from(types::BOOL),
      Token::Integer(_) => String::from(types::INT),
      Token::Float(_) => String::from(types::FLOAT),
      Token::Array(_) => String::from(types::ARRAY),
      Token::Structure { name, args: _ } => name.clone(),
      Token::Return(_) => String::from(types::RETURN),
      Token::Function(_) => String::from(types::FUNCTION),
      Token::Error(_) => String::from(types::ERROR),
      _ => String::from(types::NULL),
    },
    None => String::from(types::NULL),
  }
}

pub mod types {
  pub type Type = &'static str;
  pub const DECLARE: Type = "let";
  pub const IF: Type = "if";
  pub const FOR: Type = "for";
  pub const WHILE: Type = "while";
  pub const RETURN: Type = "return";
  pub const BREAK: Type = "break";
  pub const CONTINUE: Type = "continue";
  pub const TRUE: Type = "true";
  pub const FALSE: Type = "false";
  pub const FLOAT: Type = "float";
  pub const INT: Type = "int";
  pub const BOOL: Type = "bool";
  pub const ANY: Type = "any";
  pub const NULL: Type = "null";
  pub const STRING: Type = "String";
  pub const ARRAY: Type = "Array";
  pub const STRUCT: Type = "struct";
  pub const FUNCTION: Type = "fun";
  pub const THIS: Type = "this";
  pub const ERROR: Type = "err";
}
