mod ct;

use ct::core::itp::Interpreter;
use ct::modules::filestream;
use ct::modules::iostream;
use ct::modules::string;
use ct::structs;
use ct::syntax::std::Token;

use std::env;

fn main() {
  if let Some(file_name) = env::args().nth(1) {
    match filestream::read_file(vec![Some(Token::String(file_name))]) {
      Some(file) => match file {
        Token::String(content) => {
          let mut interpreter = Interpreter::new(content);

          interpreter.append_module(iostream::IOStream);
          interpreter.append_module(filestream::FileStream);
          interpreter.append_module(string::Str);

          interpreter.append_struct("String", structs::string::string_struct());

          interpreter.execute();
        }
        Token::Error(err) => println!("{}", err),
        _ => (),
      },
      _ => (),
    }
  } else {
    println!("Command line interpreter is not yet implemented.");
  }
}
