use std::ops::Range;

use static_regular_grammar::RegularGrammar;

mod authority;
mod fragment;
mod path;
mod path_mut;
mod query;
mod reference;

pub use authority::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use reference::*;

/// IRI.
///
/// Wrapper around a borrowed bytes slice representing an IRI.
/// An IRI can be seen as an IRI-reference with a defined [`Scheme`].
/// All methods of [`IriRef`] are available from this type, however the [`scheme`](Iri::scheme) method
/// is redefined to always return some scheme.
///
/// # Example
///
/// ```rust
/// use iref::iri::{Iri, Scheme, Authority, Path, Query, Fragment};
/// # fn main() -> Result<(), iref::iri::InvalidIri<&'static str>> {
/// let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#fragment")?;
///
/// assert_eq!(iri.scheme(), Scheme::new(b"https").unwrap());
/// assert_eq!(iri.authority(), Some(Authority::new("www.rust-lang.org").unwrap()));
/// assert_eq!(iri.path(), Path::new("/foo/bar").unwrap());
/// assert_eq!(iri.query(), Some(Query::new("query").unwrap()));
/// assert_eq!(iri.fragment(), Some(Fragment::new("fragment").unwrap()));
/// #
/// # Ok(())
/// # }
/// ```
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI",
	cache = "automata/iri.aut.cbor"
)]
#[grammar(sized(IriBuf))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Iri(str);

pub struct IriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl Iri {
	// pub fn parts(&self) -> IriParts {
	// 	pub enum State {
	// 		Scheme,
	// 		FirstSlash {
	// 			scheme_end: usize,
	// 			authority_or_path_start: usize
	// 		},
	// 		SecondSlash {
	// 			scheme_end: Option<usize>,
	// 			authority_or_path_start: usize
	// 		},
	// 		Authority {
	// 			scheme_end: Option<usize>,
	// 			authority_start: usize
	// 		},
	// 		Path {
	// 			scheme_end: Option<usize>,
	// 			authority_range: Option<Range<usize>>,
	// 			path_start: usize
	// 		},
	// 		Query {
	// 			scheme_end: Option<usize>,
	// 			authority_range: Option<Range<usize>>,
	// 			path_range: Range<usize>,
	// 			query_start: usize
	// 		},
	// 		Fragment {
	// 			scheme_end: Option<usize>,
	// 			authority_range: Option<Range<usize>>,
	// 			path_range: Range<usize>,
	// 			query_range: Option<Range<usize>>,
	// 			fragment_start: usize
	// 		}
	// 	}

	// 	let mut q = State::Scheme;
	// 	for (i, c) in self.0.char_indices() {
	// 		q = match q {
	// 			State::Scheme => match c {
	// 				':' => State::FirstSlash {
	// 					scheme_end: i,
	// 					authority_or_path_start: i + 1
	// 				},
	// 				_ => State::Scheme
	// 			}
	// 			State::FirstSlash { scheme_end, authority_or_path_start } => match c {
	// 				'/' => State::SecondSlash {
	// 					scheme_end: Some(scheme_end),
	// 					authority_or_path_start
	// 				},
	// 				'?' => State::Query {
	// 					scheme_end: Some(scheme_end),
	// 					authority_range: None,
	// 					path_range: authority_or_path_start..i,
	// 					query_start: i + 1
	// 				},
	// 				'#' => State::Fragment {
	// 					scheme_end: Some(scheme_end),
	// 					authority_range: None,
	// 					path_range: authority_or_path_start..i,
	// 					query_range: None,
	// 					fragment_start: i + 1
	// 				},
	// 				_ => State::Path {
	// 					scheme_end: Some(scheme_end),
	// 					authority_range: None,
	// 					path_start: authority_or_path_start
	// 				}
	// 			}
	// 			State::SecondSlash { scheme_end, authority_or_path_start } => match c {
	// 				'/' => State::Authority {
	// 					scheme_end,
	// 					authority_start: authority_or_path_start
	// 				},
	// 				'?' => State::Query {
	// 					scheme_end,
	// 					authority_range: None,
	// 					path_range: authority_or_path_start..i,
	// 					query_start: i + 1
	// 				},
	// 				'#' => State::Fragment {
	// 					scheme_end,
	// 					authority_range: None,
	// 					path_range: authority_or_path_start..i,
	// 					query_range: None,
	// 					fragment_start: i + 1
	// 				},
	// 				_ => State::Path {
	// 					scheme_end,
	// 					authority_range: None,
	// 					path_start: authority_or_path_start
	// 				}
	// 			}
	// 			State::Authority { scheme_end, authority_start } => match c {
	// 				'/' => State::Path {
	// 					scheme_end,
	// 					authority_range: Some(authority_start..i),
	// 					path_start: i
	// 				},
	// 				_ => State::Authority {
	// 					scheme_end,
	// 					authority_start
	// 				}
	// 			}
	// 			State::Path { scheme_end, authority_range, path_start } => match c {
	// 				'?' => State::Query {
	// 					scheme_end,
	// 					authority_range,
	// 					path_range: path_start..i,
	// 					query_start: i + 1
	// 				},
	// 				'#' => State::Fragment {
	// 					scheme_end,
	// 					authority_range,
	// 					path_range: path_start..i,
	// 					query_range: None,
	// 					fragment_start: i + 1
	// 				},
	// 				_ => State::Path {
	// 					scheme_end,
	// 					authority_range,
	// 					path_start
	// 				}
	// 			}
	// 			State::Query { scheme_end, authority_range, path_range, query_start } => match c {
	// 				'#' => State::Fragment {
	// 					scheme_end,
	// 					authority_range,
	// 					path_range,
	// 					query_range: Some(query_start..i),
	// 					fragment_start: i + 1
	// 				},
	// 				_ => State::Query { scheme_end, authority_range, path_range, query_start }
	// 			}
	// 			fragment => fragment
	// 		}
	// 	}

	// 	match state {
	// 		State::Scheme => unreachable!(),
	// 		State::
	// 	}
	// }

	/// Returns the scheme of the IRI.
	///
	/// Contrarily to [`IriRef`], the scheme of an IRI is always defined.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		unsafe {
			// SAFETY: IRIs always have a scheme.
			Scheme::new_unchecked(self.0.split(':').next().unwrap().as_bytes())
		}
	}

	/// Returns the authority part of the IRI, if any.
	pub fn authority(&self) -> Option<&Authority> {
		#[derive(Clone, Copy)]
		pub enum State {
			Scheme,
			FirstSlash,
			SecondSlash,
			Capture(usize, usize),
		}

		let mut q = State::Scheme;
		for (i, c) in self.0.char_indices() {
			q = match q {
				State::Scheme => match c {
					':' => State::FirstSlash,
					_ => State::Scheme,
				},
				State::FirstSlash => match c {
					'/' => State::SecondSlash,
					_ => break,
				},
				State::SecondSlash => match c {
					'/' => State::Capture(i + 1, i + 1),
					_ => break,
				},
				State::Capture(start, _) => match c {
					'/' | '?' | '#' => break,
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

	/// Returns the path of the IRI.
	pub fn path(&self) -> &Path {
		#[derive(Clone, Copy)]
		pub enum State {
			Scheme,
			FirstSlash(usize),
			SecondSlash(usize, usize),
			Authority(usize),
			Capture(usize, usize),
		}

		let mut q = State::Scheme;
		for (i, c) in self.0.char_indices() {
			q = match q {
				State::Scheme => match c {
					':' => State::FirstSlash(i + 1),
					_ => State::Scheme,
				},
				State::FirstSlash(_) => match c {
					'/' => State::SecondSlash(i, i + 1),
					'?' | '#' => break,
					_ => State::Capture(i, i),
				},
				State::SecondSlash(start, end) => match c {
					'/' => State::Authority(i + 1),
					'?' | '#' => break,
					_ => State::Capture(start, end),
				},
				State::Authority(_) => match c {
					'/' => State::Capture(i, i + 1),
					'?' | '#' => break,
					c => State::Authority(c.len_utf8()),
				},
				State::Capture(start, _) => match c {
					'?' | '#' => break,
					_ => State::Capture(start, i + 1),
				},
			}
		}

		let (start, end) = match q {
			State::Scheme => unreachable!(),
			State::FirstSlash(start) => (start, start),
			State::SecondSlash(start, end) => (start, end),
			State::Authority(start) => (start, start),
			State::Capture(start, end) => (start, end),
		};

		unsafe { Path::new_unchecked(&self.0[start..end]) }
	}

	pub fn query(&self) -> Option<&Query> {
		pub enum State {
			Before,
			Capture(usize, usize),
		}

		let mut q = State::Before;
		for (i, c) in self.0.char_indices() {
			q = match q {
				State::Before => match c {
					'?' => State::Capture(i + 1, i + 1),
					'#' => break,
					_ => State::Before,
				},
				State::Capture(start, _) => match c {
					'#' => break,
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
		self.0
			.split_once('#')
			.map(|(_, fragment)| unsafe { Fragment::new_unchecked(fragment) })
	}
}
