use ct::utils::module::Module;
use ct::utils::predefs::Predefs;
use ct::syntax::std::*;

use std::io::prelude::*;
use std::fs::File;

pub struct FileStream;

impl Module for FileStream {
	fn extend(&self, predefs: &mut Predefs) {
		predefs.insert(String::from("read_file"), Box::new(read_file));
	}
}

pub fn read_file(args: Vec<CtToken>) -> CtToken {
	if args.len() != 1 || args[0].is_none() { return Some(Token::Error(String::from("Not enough arguments provided"))); }
	if let Token::String(file_name) = args[0].as_ref().unwrap() {
		let file = File::open(file_name);
		match file {
			Ok(mut f) => {
				let mut contents = String::new();
				match f.read_to_string(&mut contents) {
					Ok(_) => Some(Token::String(contents)),
					Err(e) => Some(Token::Error(e.to_string()))
				}
			},
			Err(e) => Some(Token::Error(e.to_string()))
		}
	} else {
		Some(Token::Error(format!("Expected file name to be of type string, but received: {}", args[0].as_ref().unwrap())))
	}
}