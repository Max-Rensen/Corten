use ct::utils::module::Module;
use ct::utils::predefs::Predefs;
use ct::syntax::std::*;

pub struct Str;

impl Module for Str {
	fn extend(&self, predefs: &mut Predefs) {
		predefs.insert(String::from("len"), Box::new(len));
	}
}

pub fn len(args: Vec<CtToken>) -> CtToken {
	if args.len() != 1 || args[0].is_none() { return Some(Token::Error(String::from("Not enough arguments provided"))); }
	if let Token::String(s) = args[0].as_ref().unwrap() {
        Some(Token::Integer(s.len() as i32))
    } else {
		Some(Token::Error(format!("Expected structure that has function length, but received: {}", args[0].as_ref().unwrap())))
	}
}