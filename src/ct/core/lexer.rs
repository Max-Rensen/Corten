use ct::core::input::Input;
use ct::syntax::std::CtToken;
use ct::syntax::std::Token;

pub struct Lexer {
  input: Input,
  current: CtToken,
  peeked: bool,
}

impl Lexer {
  pub fn new(code: String) -> Lexer {
    Lexer {
      input: Input::new(code),
      current: None,
      peeked: false,
    }
  }

  pub fn next(&mut self) -> CtToken {
    if self.peeked {
      self.peeked = false;
      return self.current.clone();
    }

    if self.input.eof() {
      return Lexer::null();
    }
    self.read_while(Lexer::whitespace);

    let c = self.input.peek();

    if c == '"' {
      Some(Token::String(self.read_string()))
    } else if Lexer::id_start(c) {
      Some(Token::Identifier(self.read_while(Lexer::id)))
    } else if Lexer::punc(c) {
      Some(Token::Punctuation(self.input.next()))
    } else if Lexer::oper(c) {
      Some(Token::Operator(self.read_while(Lexer::oper)))
    } else if Lexer::digit(c) {
      let num = self.read_while(Lexer::number);
      if num.contains('.') {
        Some(Token::Float(match num.parse::<f64>() {
          Ok(value) => value,
          Err(_) => {
            self.error("Expected a float");
            0.0
          }
        }))
      } else {
        Some(Token::Integer(match num.parse::<i32>() {
          Ok(value) => value,
          Err(_) => {
            self.error("Expected an integer");
            0
          }
        }))
      }
    } else {
      self.error(&format!("Cannot identify: {}", c))
    }
  }

  pub fn peek(&mut self) -> CtToken {
    if self.peeked {
      return self.current.clone();
    } else {
      self.current = self.next();
      self.peeked = true;
      return self.current.clone();
    }
  }

  pub fn error(&self, s: &str) -> CtToken {
    self.input.error(s);
    Lexer::null()
  }

  pub fn null() -> CtToken {
    None
  }

  pub fn skip_line(&mut self) {
    self.read_while(|c| c != '\n');
    self.peeked = false;
  }

  // Read while functions

  fn read_while(&mut self, f: fn(char) -> bool) -> String {
    let mut result = String::new();
    while !self.input.eof() && f(self.input.peek()) {
      result.push(self.input.next());
    }

    result
  }

  fn read_string(&mut self) -> String {
    let mut esc = false;
    let mut s = String::new();
    self.input.next();

    while !self.input.eof() {
      let c = self.input.next();
      if esc {
        match c {
          'n' => s.push('\n'),
          't' => s.push('\t'),
          'r' => s.push('\r'),
          '0' => s.push('\0'),
          'v' => s.push('\x0B'),
          _ => s.push(c),
        }
        esc = false;
      } else if c == '\\' {
        esc = true;
      } else if c == '"' {
        break;
      } else {
        s.push(c)
      };
    }

    s
  }

  fn whitespace(c: char) -> bool {
    String::from(" \t\r\n").contains(c)
  }

  fn number(c: char) -> bool {
    Lexer::digit(c) || c == '.'
  }

  fn digit(c: char) -> bool {
    c >= '0' && c <= '9'
  }

  fn id(c: char) -> bool {
    Lexer::id_start(c) || Lexer::digit(c) || c == '-'
  }

  fn id_start(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
  }

  fn punc(c: char) -> bool {
    String::from("(){}[];,.").contains(c)
  }

  fn oper(c: char) -> bool {
    String::from("&|%*/+-=<>").contains(c)
  }
}
