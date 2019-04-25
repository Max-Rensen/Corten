use ct::core::parser::Parser;
use ct::env::Environment;
use ct::syntax::std::*;
use ct::utils::module::Module;
use ct::utils::predefs::Predefs;
use ct::utils::structure::Struct;
use ct::utils::structure::Structures;

pub struct Interpreter {
  parser: Parser,
  environments: Vec<Environment>,
  structs: Structures,
  predefs: Predefs,
}

impl Interpreter {
  pub fn new(code: String) -> Interpreter {
    Interpreter {
      parser: Parser::new(code),
      environments: Vec::new(),
      structs: Structures::new(),
      predefs: Predefs::new(),
    }
  }

  pub fn execute(&mut self) {
    let mut token = self.parser.next();
    let mut environments = vec![Environment::new()];

    while token.is_some() {
      self.evaluate(token, &mut environments);
      token = self.parser.next();
    }

    self.environments = environments;
  }

  pub fn append_module(&mut self, module: impl Module) {
    module.extend(&mut self.predefs);
  }

  pub fn append_struct(&mut self, name: &str, structure: Struct) {
    self.structs.append(name, structure);
  }

  fn evaluate(&self, raw: CtToken, environments: &mut Vec<Environment>) -> CtToken {
    match raw {
      Some(token) => match token {
        Token::Integer(_)
        | Token::Float(_)
        | Token::Boolean(_)
        | Token::String(_)
        | Token::Return(_)
        | Token::Break
        | Token::Continue => Some(token),
        Token::VariableCall(name) => match Environment::lookup(environments, &name) {
          Some(environment) => environment.get(&name),
          None => self
            .parser
            .error(&format!("Unable to find variable: {}", name)),
        },

        Token::Binary {
          operator,
          left,
          right,
        } => {
          if left.is_none() || right.is_none() {
            return self.parser.error(&format!(
              "Unable to parse binary expression: {} {} {}",
              left.unwrap(),
              operator,
              right.unwrap()
            ));
          }

          if operator == "=" {
            let left = left.unwrap();
            return match left {
              Token::Variable {
                name,
                return_type: _,
              } => {
                let right = self.evaluate(*right, environments);
                environments.last_mut().unwrap().define(name, right)
              }

              Token::VariableCall(name) => {
                let right = self.evaluate(*right, environments);
                match Environment::lookup(environments, &name) {
                  Some(mut environment) => environment.set(&name, right),
                  None => self.parser.error(&format!("Unknown variable: {}", name)),
                }
              }

              _ => self.parser.error(&format!(
                "Unable to assign right hand value to left hand variable: {} = {}",
                left,
                right.unwrap()
              )),
            };
          } else {
            self.apply_binary(
              operator,
              self.evaluate(*left, environments),
              self.evaluate(*right, environments),
            )
          }
        }

        Token::Function(Function { header, body }) => {
          if Environment::lookup(environments, &header.name).is_some() {
            self
              .parser
              .error(&format!("Function already exists: {}", header.name))
          } else {
            environments.last_mut().unwrap().define(
              header.name.clone(),
              Some(Token::Function(Function { header, body })),
            )
          }
        }

        Token::FunctionCall { name, args } => {
          if self.predefs.contains(&name) {
            let mut eval_args = Vec::new();
            for arg in args {
              eval_args.push(match arg {
                None => None,
                Some(value) => self.evaluate(Some(value), environments),
              });
            }

            self.predefs.execute(&name, eval_args)
          } else {
            let function = match Environment::lookup(environments, &name) {
              Some(e) => e.get(&name),
              None => self.parser.error(&format!("Unknown function: {}", name)),
            };

            self.execute_function(&name, function, &args, environments)
          }
        }

        Token::If(ifs) => {
          for i in ifs.iter() {
            let condition = self.evaluate((*i.condition).clone(), environments);
            if let Some(value) = condition {
              if let Token::Boolean(b) = value {
                if b {
                  let mut scope = Environment::new();
                  environments.push(scope);
                  for raw in i.body.iter() {
                    let t = self.evaluate((*raw).clone(), environments);
                    if t.is_none() {
                      continue;
                    }
                    let t = t.unwrap();
                    if let Token::Break = &t {
                      return Some(Token::Break);
                    } else if let Token::Continue = &t {
                      return Some(Token::Continue);
                    }
                    let maybe_return = self.unwrap_return(Some(t), environments);
                    if maybe_return.is_some() {
                      return maybe_return;
                    }
                  }
                  break;
                } else {
                  continue;
                }
              } else {
                return self.parser.error(&format!(
                  "Expected boolean expression inside if statement, but received: {}",
                  value
                ));
              }
            }
          }

          None
        }

        Token::While { condition, body } => {
          let cond = self.evaluate((*condition).clone(), environments);
          if cond.is_some() {
            let mut cond = cond.unwrap();
            while let Token::Boolean(true) = cond {
              match {
                let scope = Environment::new();
                environments.push(scope);
                for raw in body.iter() {
                  let t = self.evaluate((*raw).clone(), environments);
                  if t.is_none() {
                    continue;
                  }
                  let t = t.unwrap();
                  if let Token::Break = t {
                    return None;
                  } else if let Token::Continue = t {
                    break;
                  }
                }

                None
              } {
                Some(value) => return Some(value),
                None => (),
              }

              cond = self.evaluate((*condition).clone(), environments).unwrap();
            }
          }

          None
        }

        Token::StructureCall {
          name,
          attribute,
          args,
        } => {
          let mut var = None;
          let structure = self
            .structs
            .get(&match Environment::lookup(environments, &name) {
              Some(environment) => {
                var = environment.get(&name);
                get_type(&var)
              }
              None => {
                self
                  .parser
                  .error(&format!("Unable to find variable: {}", name));
                String::from(types::NULL)
              }
            });

          let value = structure.get(&attribute);
          if value.is_some() {
            if args.is_some() {
              let mut real_args = vec![var];
              real_args.extend_from_slice(&args.unwrap());
              self.execute_function(&attribute, value, &real_args, environments)
            } else {
              value
            }
          } else {
            None
          }
        }

        _ => self.parser.error(&format!("Unable to evaluate: {}", token)),
      },
      None => self.parser.error("Unable to evaluate: None"),
    }
  }

  fn execute_function(
    &self,
    name: &String,
    function: CtToken,
    args: &Vec<CtToken>,
    environments: &mut Vec<Environment>,
  ) -> CtToken {
    match function.unwrap() {
      Token::Function(Function { header, body }) => {
        let mut scope = Environment::new();
        if args.len() != header.args.len() {
          self.parser.error("Expected the amount of function parameters to be the same as the amount of given arguments");
        }

        for i in 0..args.len() {
          let t = self.evaluate(args[i].clone(), environments);

          // TODO: Handle possible type differences
          let name = String::from(
            if let Token::Variable {
              name,
              return_type: _,
            } = &header.args[i].as_ref().unwrap()
            {
              name
            } else {
              ""
            },
          );
          scope.define(name, t);
        }

        environments.push(scope);
        for raw in body {
          let t = self.evaluate(raw, environments);
          if t.is_none() {
            continue;
          }
          let maybe_return = self.unwrap_return(t, environments);
          if maybe_return.is_some() {
            return maybe_return;
          }
        }

        None
      }
      _ => {
        return self
          .parser
          .error(&format!("Unable to execute function: {}", name));
      }
    }
  }

  fn apply_binary(&self, operator: String, left: CtToken, right: CtToken) -> CtToken {
    if left.is_none() || right.is_none() {
      return self.parser.error(&format!(
        "Unable to parse binary expression: {} {} {}",
        left.unwrap(),
        operator,
        right.unwrap()
      ));
    }
    let left = left.as_ref().unwrap();
    let right = right.as_ref().unwrap();

    if let (Token::Integer(num1), Token::Integer(num2)) = (left, right) {
      match operator.as_ref() {
        "<" => return Some(Token::Boolean(num1 < num2)),
        ">" => return Some(Token::Boolean(num1 > num2)),
        "<=" => return Some(Token::Boolean(num1 <= num2)),
        ">=" => return Some(Token::Boolean(num1 >= num2)),
        "==" => return Some(Token::Boolean(num1 == num2)),
        "!=" => return Some(Token::Boolean(num1 != num2)),
        "+" => return Some(Token::Integer(num1 + num2)),
        "-" => return Some(Token::Integer(num1 - num2)),
        "*" => return Some(Token::Integer(num1 * num2)),
        "/" => return Some(Token::Integer(num1 / num2)),
        "%" => return Some(Token::Integer(num1 % num2)),
        _ => (),
      }
    }

    let mut maybe_num1 = None;
    let mut maybe_num2 = None;

    if let (Token::Float(num1), Token::Float(num2)) = (left, right) {
      maybe_num1 = Some(*num1);
      maybe_num2 = Some(*num2);
    }

    if let (Token::Integer(num1), Token::Float(num2)) = (left, right) {
      maybe_num1 = Some(*num1 as f64);
      maybe_num2 = Some(*num2);
    }

    if let (Token::Float(num1), Token::Integer(num2)) = (left, right) {
      maybe_num1 = Some(*num1);
      maybe_num2 = Some(*num2 as f64);
    }

    if maybe_num1.is_some() && maybe_num2.is_some() {
      match operator.as_ref() {
        "<" => return Some(Token::Boolean(maybe_num1.unwrap() < maybe_num2.unwrap())),
        ">" => return Some(Token::Boolean(maybe_num1.unwrap() > maybe_num2.unwrap())),
        "<=" => return Some(Token::Boolean(maybe_num1.unwrap() <= maybe_num2.unwrap())),
        ">=" => return Some(Token::Boolean(maybe_num1.unwrap() >= maybe_num2.unwrap())),
        "==" => return Some(Token::Boolean(maybe_num1.unwrap() == maybe_num2.unwrap())),
        "!=" => return Some(Token::Boolean(maybe_num1.unwrap() != maybe_num2.unwrap())),
        "+" => return Some(Token::Float(maybe_num1.unwrap() + maybe_num2.unwrap())),
        "-" => return Some(Token::Float(maybe_num1.unwrap() - maybe_num2.unwrap())),
        "*" => return Some(Token::Float(maybe_num1.unwrap() * maybe_num2.unwrap())),
        "/" => return Some(Token::Float(maybe_num1.unwrap() / maybe_num2.unwrap())),
        "%" => return Some(Token::Float(maybe_num1.unwrap() % maybe_num2.unwrap())),
        _ => (),
      }
    }

    if let (Token::Boolean(bool1), Token::Boolean(bool2)) = (left, right) {
      match operator.as_ref() {
        "==" => return Some(Token::Boolean(bool1 == bool2)),
        "!=" => return Some(Token::Boolean(bool1 != bool2)),
        "&&" => return Some(Token::Boolean(*bool1 && *bool2)),
        "||" => return Some(Token::Boolean(*bool1 || *bool2)),
        _ => (),
      }
    }

    if let (Token::String(str1), Token::String(str2)) = (left, right) {
      match operator.as_ref() {
        "<" => return Some(Token::Boolean(str1 < str2)),
        ">" => return Some(Token::Boolean(str1 > str2)),
        "<=" => return Some(Token::Boolean(str1 <= str2)),
        ">=" => return Some(Token::Boolean(str1 >= str2)),
        "==" => return Some(Token::Boolean(str1 == str2)),
        "!=" => return Some(Token::Boolean(str1 != str2)),
        "+" => return Some(Token::String((*str1).clone() + str2)),
        _ => (),
      }
    }

    self.parser.error(&format!(
      "Unknown operator expression: {} {} {}",
      left, operator, right
    ))
  }

  fn unwrap_return(&self, t: CtToken, environments: &mut Vec<Environment>) -> CtToken {
    if let Token::Return(value) = t.unwrap() {
      let return_value = self.evaluate(*value, environments);

      // TODO: Handle possible type differences
      let return_value = self.evaluate(return_value, environments);
      environments.pop();
      return_value
    } else {
      None
    }
  }

  fn string_to_int(string: &String) -> Option<i32> {
    if let Ok(i) = string.parse::<i32>() {
      Some(i)
    } else {
      None
    }
  }
}
