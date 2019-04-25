use std::collections::HashMap;

use ct::syntax::std::*;
use ct::utils::structure::Struct;

pub fn string_struct() -> Struct {
  let mut s = Struct {
    constructor: Function {
      header: FunctionHeader {
        name: String::from("constructor"),
        args: Vec::new(),
        return_type: types::THIS,
      },
      body: Vec::new(),
    },
    prototype: HashMap::new(),
  };

  s.prototype.insert(
    String::from("len"),
    Some(Token::Function(Function {
      header: FunctionHeader {
        name: String::from("len"),
        args: vec![Some(Token::Variable {
          name: String::from("s"),
          return_type: types::STRING,
        })],
        return_type: types::INT,
      },
      body: vec![Some(Token::Return(Box::new(Some(Token::FunctionCall {
        name: String::from("len"),
        args: vec![Some(Token::VariableCall(String::from("s")))],
      }))))],
    })),
  );

  s
}
