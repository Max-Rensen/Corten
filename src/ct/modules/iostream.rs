use ct::utils::module::Module;
use ct::utils::predefs::Predefs;
use ct::syntax::std::*;

use std::io;
use std::io::Write;

pub fn print(args: Vec<CtToken>) -> CtToken {
	if args.len() == 0 { return None; }
        match args[0].as_ref() {
            Some(t) => 
                match t {
                    Token::String(value) => {
                        let mut res = String::new();
                        let mut index = 1;
                        let mut open = false;
                        let mut escaped = false;
                        for c in value.chars() {
                            match c {
                                '{' => {
                                    if !escaped { 
                                        open = true;
                                        escaped = true;
                                    } else {
                                        open = false;
                                        escaped = false;
                                        res.push('{');
                                    }
                                },
                                '}' => 
                                    if open && escaped {
                                        if index < args.len() && !args[index].is_none() {
                                            res += &format!("{}", args[index].as_ref().unwrap());
                                            index += 1;
                                        }
                                        open = false;
                                        escaped = false;
                                    } else if !open && escaped {
                                        res.push('}');
                                    } else {
                                        escaped = true;
                                    },
                                _ => {
                                    res.push(c);
                                    escaped = false;
                                    open = false;
                                }
                            }
                        }
                        print!("{}", res);
                    },
                    _ => print!("{}", t)
                },
            None => print!("None")
    }
	None
}

pub fn input(args: Vec<CtToken>) -> CtToken {
	print(args);

	let mut input = String::new();
	io::stdin().read_line(&mut input).expect("Unable to read input");
	Some(Token::String(String::from(input.trim())))
}

pub fn flush(args: Vec<CtToken>) -> CtToken {
    print(args);
    io::stdout().flush().ok().expect("Unable not flush stdout");
    None
}

pub struct IOStream;

impl Module for IOStream {
	fn extend(&self, predefs: &mut Predefs) {
		predefs.insert(String::from("print"), Box::new(print));
		predefs.insert(String::from("input"), Box::new(input));
        predefs.insert(String::from("flush"), Box::new(flush));
	} 
}