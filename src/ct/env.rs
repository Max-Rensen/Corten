use ct::syntax::std::*;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Environment {
  pub vars: HashMap<String, CtToken>,
}

impl Environment {
  pub fn new() -> Environment {
    Environment {
      vars: HashMap::new(),
    }
  }

  pub fn lookup<'a>(
    environments: &'a mut Vec<Environment>,
    name: &String,
  ) -> Option<&'a mut Environment> {
    for env in environments.iter_mut().rev() {
      if env.vars.contains_key(name) {
        return Some(env);
      }
    }

    None
  }

  pub fn get(&self, name: &String) -> CtToken {
    self.vars[name].clone()
  }

  pub fn set(&mut self, name: &String, value: CtToken) -> CtToken {
    if self.vars.contains_key(name) {
      self.define(name.clone(), value)
    } else {
      None
    }
  }

  pub fn define(&mut self, name: String, value: CtToken) -> CtToken {
    self.vars.insert(name.clone(), value);
    self.get(&name)
  }
}
