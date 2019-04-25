use std::collections::HashMap;

use ct::syntax::std::CtToken;
use ct::syntax::std::*;

#[derive(Debug, Clone)]
pub struct Struct {
  pub constructor: Function,
  pub prototype: HashMap<String, CtToken>,
}

impl Struct {
  pub fn new(constructor: Option<Function>, prototype: HashMap<String, CtToken>) -> Struct {
    Struct {
      constructor: if constructor.is_some() {
        let unwraped = constructor.unwrap();
        Function {
          header: FunctionHeader {
            name: String::from("constructor"),
            args: unwraped.header.args,
            return_type: types::THIS,
          },
          body: unwraped.body,
        }
      } else {
        Function {
          header: FunctionHeader {
            name: String::from("constructor"),
            args: Vec::new(),
            return_type: types::THIS,
          },
          body: Vec::new(),
        }
      },
      prototype: prototype,
    }
  }

  pub fn get(&self, name: &String) -> CtToken {
    if self.prototype.contains_key(name) {
      self.prototype[name].clone()
    } else {
      None
    }
  }

  pub fn set(&mut self, name: &String, value: CtToken) -> CtToken {
    self.prototype.insert(name.clone(), value);
    self.get(name)
  }
}

pub struct Structures {
  structs: HashMap<String, Struct>,
}

impl Structures {
  pub fn new() -> Structures {
    Structures {
      structs: HashMap::new(),
    }
  }

  pub fn append(&mut self, name: &str, structure: Struct) {
    self.structs.insert(String::from(name), structure);
  }

  pub fn get(&self, name: &str) -> &Struct {
    &self.structs[name]
  }
}
