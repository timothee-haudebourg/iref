/// Percent-Encoded string
pub struct PctStr<'a> {
	data: &str
}

impl<'a> PctStr<'a> {
	pub fn new<S: AsRef<str>>(str: &S) -> Result<PctStr<'a>, ()> {
		// TODO
	}

	/// Get the underlying Percent-Encoded string.
	pub fn as_str(&self) -> &str {
		self.data
	}

	pub fn decode(&self) -> String {
		// TODO
	}

	pub fn chars(&self) -> Chars {
		// TODO
	}
}

pub struct PctString {
	data: String
}

// pub trait Encoder {
// 	// ...
// }
//
// pub struct QueryEncoder {
// 	// ...
// }

impl PctString {
	pub fn new<S: AsRef<str>>(str: &S) -> Result<PctString, ()> {
		// TODO
	}

	/// Encode a string.
	pub fn encode<S: AsRef<str>>(str: &S) -> PctString {
		// TODO
	}

	/// Get the underlying Percent-Encoded string.
	pub fn as_str(&self) -> &str {
		self.data
	}

	pub fn decode(&self) -> String {
		// TODO
	}

	pub fn chars(&self) -> Chars {
		// TODO
	}
}
