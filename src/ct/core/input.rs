pub struct Input {
	code: Vec<char>,
	index: u64,
	line: u32,
	col: u32,
}

impl Input {
	pub fn new(code: String) -> Input {
		Input {
			code: code.chars().collect(),
			index: 0,
			line: 1,
			col: 0,
		}
	}

	pub fn next(&mut self) -> char {
		let c = self.code[self.index as usize];

		if c == '\n' {
			self.line += 1;
			self.col = 0;
		} else {
			self.col += 1;
		}

		if self.index < self.code.len() as u64 {
			self.index += 1;
		}

		c
	}

	pub fn peek(&self) -> char {
		self.code[self.index as usize]
	}

	pub fn eof(&self) -> bool {
		self.index as usize >= self.code.len()
	}

	pub fn error(&self, message: &str) {
		panic!(format!("Error at (line: {}, col: {}): \n{}\n", self.line, self.col, message));
	}
}