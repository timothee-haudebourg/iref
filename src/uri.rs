use static_regular_grammar::RegularGrammar;

mod authority;
mod fragment;
mod query;
mod scheme;

pub use authority::*;
pub use fragment::*;
pub use query::*;
pub use scheme::*;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "URI",
	ascii,
	cache = "automata/uri.aut.cbor"
)]
#[grammar(sized(UriBuf))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Uri([u8]);

impl Uri {
	/// Returns the scheme of the URI.
	///
	/// Contrarily to [`UriRef`], the scheme of an URI is always defined.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		unsafe {
			// SAFETY: URIs always have a scheme.
			Scheme::new_unchecked(self.0.split(|b| *b == b':').next().unwrap())
		}
	}

	/// Returns the authority part of the URI, if any.
	pub fn authority(&self) -> Option<&Authority> {
		#[derive(Clone, Copy)]
		pub enum State {
			Scheme,
			FirstSlash,
			SecondSlash,
			Capture(usize, usize),
		}

		let mut q = State::Scheme;
		for (i, c) in self.0.iter().copied().enumerate() {
			q = match q {
				State::Scheme => match c {
					b':' => State::FirstSlash,
					_ => State::Scheme,
				},
				State::FirstSlash => match c {
					b'/' => State::SecondSlash,
					_ => break,
				},
				State::SecondSlash => match c {
					b'/' => State::Capture(i + 1, i + 1),
					_ => break,
				},
				State::Capture(start, _) => match c {
					b'/' | b'?' | b'#' => break,
					_ => State::Capture(start, i + 1),
				},
			}
		}

		match q {
			State::Capture(start, end) => {
				Some(unsafe { Authority::new_unchecked(&self.0[start..end]) })
			}
			_ => None,
		}
	}

	// /// Returns the path of the URI.
	// pub fn path(&self) -> &Path {
	// 	#[derive(Clone, Copy)]
	// 	pub enum State {
	// 		Scheme,
	// 		FirstSlash(usize),
	// 		SecondSlash(usize, usize),
	// 		Authority(usize),
	// 		Capture(usize, usize)
	// 	}

	// 	let mut q = State::Scheme;
	// 	for (i, c) in &self.0 {
	// 		q = match q {
	// 			State::Scheme => match c {
	// 				b':' => State::FirstSlash(i + 1),
	// 				_ => State::Scheme
	// 			}
	// 			State::FirstSlash(_) => match c {
	// 				b'/' => State::SecondSlash(i, i + 1),
	// 				b'?' | b'#' => break,
	// 				_ => State::Capture(i, i)
	// 			}
	// 			State::SecondSlash(start, end) => match c {
	// 				b'/' => State::Authority(i + 1),
	// 				b'?' | b'#' => break,
	// 				_ => State::Capture(start, end)
	// 			}
	// 			State::Authority(_) => match c {
	// 				b'/' => State::Capture(i, i + 1),
	// 				b'?' | b'#' => break,
	// 				c => State::Authority(c.len_utf8())
	// 			}
	// 			State::Capture(start, _) => match c {
	// 				b'?' | b'#' => break,
	// 				_ => State::Capture(start, i + 1)
	// 			}
	// 		}
	// 	}

	// 	let (start, end) = match q {
	// 		State::Scheme => unreachable!(),
	// 		State::FirstSlash(start) => (start, start),
	// 		State::SecondSlash(start, end) => (start, end),
	// 		State::Authority(start) => (start, start),
	// 		State::Capture(start, end) => (start, end)
	// 	};

	// 	unsafe {
	// 		Path::new_unchecked(&self.0[start..end])
	// 	}
	// }

	pub fn query(&self) -> Option<&Query> {
		pub enum State {
			Before,
			Capture(usize, usize),
		}

		let mut q = State::Before;
		for (i, c) in self.0.iter().copied().enumerate() {
			q = match q {
				State::Before => match c {
					b'?' => State::Capture(i + 1, i + 1),
					b'#' => break,
					_ => State::Before,
				},
				State::Capture(start, _) => match c {
					b'#' => break,
					_ => State::Capture(start, i + 1),
				},
			}
		}

		match q {
			State::Before => None,
			State::Capture(start, end) => {
				Some(unsafe { Query::new_unchecked(&self.0[start..end]) })
			}
		}
	}

	pub fn fragment(&self) -> Option<&Fragment> {
		for i in 0..self.0.len() {
			if self.0[i] == b'#' {
				return Some(unsafe { Fragment::new_unchecked(&self.0[i + 1..]) });
			}
		}

		None
	}
}
