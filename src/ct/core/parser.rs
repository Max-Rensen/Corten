use ct::core::lexer::Lexer;
use ct::syntax::std::*;

pub struct Parser {
  lexer: Lexer,
  req_sc: bool,
}

impl Parser {
  pub fn new(code: String) -> Parser {
    Parser {
      lexer: Lexer::new(code),
      req_sc: true,
    }
  }

  pub fn next(&mut self) -> CtToken {
    let t = self.parse_primary();
    if t.is_none() {
      return Lexer::null();
    }

    if self.req_sc {
      self.skip(';');
    } else {
      self.req_sc = true;
    }

    t
  }

  fn parse_primary(&mut self) -> CtToken {
    let left = self.parse_generic();
    if left.is_none() {
      return Lexer::null();
    }

    self.parse_binary(0, left)
  }

  fn parse_generic(&mut self) -> CtToken {
    let t = self.lexer.peek();
    if t.is_none() {
      return Lexer::null();
    }

    let mut t = t.unwrap();

    while let Token::Operator(op) = t {
      if op == "//" {
        self.lexer.skip_line();
        t = match self.lexer.peek() {
          Some(value) => value,
          None => return Lexer::null(),
        };
      } else {
        t = Token::Operator(op);
        break;
      }
    }

    match t {
      Token::Identifier(id) => self.parse_identifier(id),
      Token::Integer(_) | Token::Float(_) | Token::String(_) => self.lexer.next(),
      Token::Punctuation('(') => self.parse_parenthesis(),
      _ => self.error(&format!("Unable to parse: {}", t)),
    }
  }

  fn parse_binary(&mut self, expr_prec: i8, left_in: CtToken) -> CtToken {
    if left_in.is_none() {
      return Lexer::null();
    }

    let mut left = left_in;

    loop {
      let (prec, operator) = self.get_precedence();

      if prec < expr_prec {
        return left;
      }

      let operator = operator.unwrap();
      self.lexer.next(); // Skip operator
      let mut right = self.parse_generic();

      let (new_prec, _) = self.get_precedence();
      if prec < new_prec {
        right = self.parse_binary(prec + 1, right);
        if right.is_none() {
          return Lexer::null();
        }
      }

      left = Some(Token::Binary {
        operator,
        left: Box::new(left),
        right: Box::new(right),
      });
    }
  }

  fn parse_identifier(&mut self, id: String) -> CtToken {
    self.lexer.next(); // Skip identifier

    match &id[..] {
      types::TRUE => return Some(Token::Boolean(true)),
      types::FALSE => return Some(Token::Boolean(false)),
      types::RETURN => return Some(Token::Return(Box::new(self.parse_primary()))),
      types::BREAK => return Some(Token::Break),
      types::CONTINUE => return Some(Token::Continue),
      _ => (),
    };

    if !self.equals('(') {
      return if self.equals('.') {
        self.skip('.'); // Skip punctuation
        let t = self.lexer.next().unwrap();
        let args = if self.equals('(') {
          Some(self.parse_arguments())
        } else {
          None
        };

        match t {
          Token::Identifier(s) => Some(Token::StructureCall {
            name: id,
            attribute: s.clone(),
            args,
          }),
          _ => self.error(&format!(
            "Expected structure attribute to be an identifier, but received: {}",
            t
          )),
        }
      } else if id == types::DECLARE {
        self.parse_function()
      } else {
        Some(Token::VariableCall(id))
      };
    }

    if id == types::IF {
      return self.parse_if();
    }
    if id == types::FOR {
      return self.parse_for();
    }
    if id == types::WHILE {
      return self.parse_while();
    }
    if id == types::STRUCT {
      return self.parse_struct();
    }

    return Some(Token::FunctionCall {
      name: id,
      args: self.parse_arguments(),
    });
  }

  fn parse_parenthesis(&mut self) -> CtToken {
    self.skip('(');
    let t = self.parse_primary();

    if !t.is_none() {
      self.skip(')');
      self.req_sc = true;
      return t;
    }

    Lexer::null()
  }

  fn parse_arguments(&mut self) -> Vec<CtToken> {
    self.skip('(');

    let mut args = Vec::new();
    while !self.equals(')') {
      let token = self.parse_primary();
      if token.is_none() {
        self.error(&format!("Invalid parameter in funcion arguments"));
      }

      args.push(token);
      if !self.equals(')') {
        self.skip(',');
      }
    }

    self.skip(')');
    self.req_sc = true;

    args
  }

  fn parse_function_body(&mut self) -> Vec<CtToken> {
    self.skip('{');
    let mut body = Vec::new();

    while !self.equals('}') {
      body.push(self.next());
    }

    self.skip('}');
    self.req_sc = false;

    body
  }

  fn parse_function_header(&mut self) -> CtToken {
    let name;

    match self.lexer.next() {
      Some(t) => match t {
        Token::Identifier(id) => name = id,
        _ => {
          return self.error(&format!("Expected variable name, found: {}", t));
        }
      },
      None => return Lexer::null(),
    }

    if !self.equals('(') {
      return Some(Token::Identifier(name));
    }

    Some(Token::FunctionHeader(FunctionHeader {
      name,
      args: self.parse_arguments(),
      return_type: types::ANY,
    }))
  }

  fn parse_function(&mut self) -> CtToken {
    let proto = self.parse_function_header();
    let header;

    match proto {
      Some(t) => match t {
        Token::Identifier(id) => {
          return Some(Token::Variable {
            name: id,
            return_type: types::ANY,
          });
        }
        Token::FunctionHeader(fh) => header = fh,
        _ => {
          return self.error("Unknown function header");
        }
      },
      None => {
        return Lexer::null();
      }
    }

    Some(Token::Function(Function {
      header,
      body: self.parse_function_body(),
    }))
  }

  fn parse_if(&mut self) -> CtToken {
    let mut ifs = Vec::new();
    let mut is_else = false;

    loop {
      let condition = if let Some(token) = self.lexer.peek() {
        if let Token::Identifier(s) = token {
          if s == "if" {
            self.skip('(');
            self.parse_primary()
          } else {
            self
              .lexer
              .error("Expected 'if' identifier after 'else' identifier")
          }
        } else if is_else {
          Lexer::null()
        } else {
          self.skip('(');
          self.parse_primary()
        }
      } else {
        return Lexer::null();
      };

      if !condition.is_none() {
        self.skip(')');
      }

      let body = self.parse_function_body();
      self.req_sc = true;

      ifs.push(If {
        condition: Box::new(if condition.is_none() {
          Some(Token::Boolean(true))
        } else {
          condition
        }),
        body,
        return_type: types::ANY,
      });

      is_else = if let Some(token) = self.lexer.peek() {
        if let Token::Identifier(s) = token {
          if s == "else" {
            true
          } else {
            false
          }
        } else {
          false
        }
      } else {
        false
      };

      if !is_else {
        break;
      }
      self.lexer.next();
    }

    self.req_sc = false;
    Some(Token::If(ifs))
  }

  fn parse_for(&mut self) -> CtToken {
    Lexer::null()
  }

  fn parse_while(&mut self) -> CtToken {
    self.skip('(');
    let condition = self.parse_primary();
    self.skip(')');

    Some(Token::While {
      condition: Box::new(condition),
      body: self.parse_function_body(),
    })
  }

  fn parse_struct(&mut self) -> CtToken {
    let raw = self.lexer.next();
    if let Some(s) = raw.as_ref() {
      if let Token::String(name) = s {
        self.skip('{');

        self.skip('}');
      }
    }

    self.error(&format!(
      "Expected a string after 'struct' keyword, but received {}",
      if raw.is_some() {
        raw.unwrap()
      } else {
        Token::String(String::from("None"))
      }
    ))
  }

  fn skip(&mut self, c: char) {
    if self.equals(c) {
      self.lexer.next();
    } else {
      let value = match self.lexer.peek() {
        Some(value) => value,
        None => Token::Error(String::from("None")),
      };
      self.error(&format!("Expected '{}', but received {}", c, value));
    }
  }

  fn equals(&mut self, c: char) -> bool {
    let t = self.lexer.peek();
    if !t.is_none() {
      match t.unwrap() {
        Token::Punctuation(ch) => {
          if ch == c {
            true
          } else {
            false
          }
        }
        _ => false,
      }
    } else {
      false
    }
  }

  fn get_precedence(&mut self) -> (i8, Option<String>) {
    let bin_op = self.lexer.peek();
    if bin_op.is_none() {
      return (-1, None);
    }

    match bin_op.unwrap() {
      Token::Operator(value) => (
        match &value[..] {
          "=" => 1,
          "+=" => 1,
          "-=" => 1,
          "||" => 5,
          "&&" => 6,
          "<" => 10,
          ">" => 10,
          ">=" => 10,
          "<=" => 10,
          "==" => 10,
          "!=" => 10,
          "+" => 20,
          "-" => 20,
          "*" => 30,
          "/" => 30,
          "%" => 30,
          _ => -1,
        },
        Some(value),
      ),
      _ => (-1, None),
    }
  }

  pub fn error(&self, s: &str) -> CtToken {
    self.lexer.error(s);
    Lexer::null()
  }
}
