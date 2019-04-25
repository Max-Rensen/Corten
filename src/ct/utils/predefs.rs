use ct::syntax::std::*;
use std::collections::HashMap;

pub struct Predefs {
	functions: HashMap<String, Box<Fn(Vec<CtToken>) -> CtToken>>
}

impl Predefs {
	pub fn new() -> Predefs {
		Predefs {
			functions: HashMap::new()
		}
	}

	pub fn contains(&self, name: &String) -> bool {
		self.functions.contains_key(name)
	}

	pub fn execute(&self, name: &String, args: Vec<CtToken>) -> CtToken {
		if self.contains(name) {
			self.functions[name](args)
		} else {
			None
		}
	}

	pub fn insert(&mut self, name: String, function: Box<Fn(Vec<CtToken>) -> CtToken>) {
		self.functions.insert(name, function);
	}
}