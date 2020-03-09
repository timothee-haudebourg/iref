/// Checks if a string is a correct percent-encoded string.
pub fn is_pct_encoded(str: &str) -> bool {
	let mut chars = str.chars();
	loop {
		match chars.next() {
			Some('%') => {
				match chars.next() {
					Some(c) if c.is_digit(16) => {
						match chars.next() {
							Some(c) if c.is_digit(16) => {
								break
							},
							_ => return false
						}
					},
					_ => return false
				}
			},
			Some(_) => (),
			None => break
		}
	}

	true
}

struct Chars<'a> {
	inner: std::str::Chars<'a>
}

impl<'a> Iterator for Chars<'a> {
	type Item = char;

	fn next(&mut self) -> Option<char> {
		match self.inner.next() {
			Some('%') => {
				let a = self.inner.next().unwrap().to_digit(16).unwrap();
				let b = self.inner.next().unwrap().to_digit(16).unwrap();
				let codepoint = (a << 4 | b) as u32;
				Some(unsafe { std::char::from_u32_unchecked(codepoint) });
			},
			Some(c) => Some(c),
			None => None
		}
	}
}

impl<'a> FusedIterator for Chars<'a> { }

/// Percent-Encoded string
pub struct PctStr<'a> {
	data: &str
}

impl<'a> PctStr<'a> {
	pub fn new<S: AsRef<str>>(str: &S) -> Result<PctStr<'a>, ()> {
		if is_pct_encoded(str) {
			Ok(PctStr {
				data: str.as_ref()
			})
		} else {
			Err(())
		}
	}

	/// Length of the string, in bytes.
	///
	/// Note that two percent-encoded strings with different lengths may
	/// represent the same string.
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Get the underlying Percent-Encoded string.
	pub fn as_str(&self) -> &str {
		self.data
	}

	pub fn chars(&self) -> Chars {
		Chars {
			inner: self.data.chars()
		}
	}

	pub fn decode(&self) -> String {
		let mut decoded = String::with_capacity(self.len());
		for c in self.chars() {
			decoded.push(c)
		}

		decoded
	}
}

pub struct PctString {
	data: String
}

impl PctString {
	pub fn new<S: AsRef<str>>(str: &S) -> Result<PctString, ()> {
		if is_pct_encoded(str) {
			Ok(PctString {
				data: str.as_ref().to_string()
			})
		} else {
			Err(())
		}
	}

	/// Encode a string.
	pub fn encode<S: AsRef<str>>(str: &S) -> PctString {
		panic!("TODO")
	}

	/// Get the underlying Percent-Encoded string.
	pub fn as_str(&self) -> &str {
		self.data.as_ref()
	}

	pub fn as_pct_str(&self) -> &PctStr {
		PctStr {
			data: self.data.as_ref()
		}
	}

	pub fn decode(&self) -> String {
		self.as_pct_str().decode()
	}

	pub fn chars(&self) -> Chars {
		self.as_pct_str().chars()
	}
}
