use core::fmt;
use std::ops::Range;

use static_regular_grammar::RegularGrammar;

use super::{Authority, Fragment, Path, Query, Scheme};

/// IRI reference.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI-reference",
	cache = "automata/iri/reference.aut.cbor"
)]
#[grammar(sized(IriRefBuf))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct IriRef(str);

pub struct IriRefParts<'a> {
	pub scheme: Option<&'a Scheme>,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl IriRef {
	pub fn parts(&self) -> IriRefParts {
		pub enum State {
			Start,
			SchemeOrPath,
			FirstSlash {
				scheme_end: usize,
				authority_or_path_start: usize,
			},
			SecondSlash {
				scheme_end: Option<usize>,
				authority_or_path_start: usize,
			},
			Authority {
				scheme_end: Option<usize>,
				authority_start: usize,
			},
			Path {
				scheme_end: Option<usize>,
				authority_range: Option<Range<usize>>,
				path_start: usize,
			},
			Query {
				scheme_end: Option<usize>,
				authority_range: Option<Range<usize>>,
				path_range: Range<usize>,
				query_start: usize,
			},
			Fragment {
				scheme_end: Option<usize>,
				authority_range: Option<Range<usize>>,
				path_range: Range<usize>,
				query_range: Option<Range<usize>>,
				fragment_start: usize,
			},
		}

		let mut q = State::Start;
		for (i, c) in self.0.char_indices() {
			q = match q {
				State::Start => match c {
					':' => State::FirstSlash {
						scheme_end: i,
						authority_or_path_start: i + 1,
					},
					'/' => State::SecondSlash {
						scheme_end: None,
						authority_or_path_start: i,
					},
					'?' => State::Query {
						scheme_end: None,
						authority_range: None,
						path_range: 0..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end: None,
						authority_range: None,
						path_range: 0..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::SchemeOrPath,
				},
				State::SchemeOrPath => match c {
					':' => State::FirstSlash {
						scheme_end: i,
						authority_or_path_start: i + 1,
					},
					'?' => State::Query {
						scheme_end: None,
						authority_range: None,
						path_range: 0..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end: None,
						authority_range: None,
						path_range: 0..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::SchemeOrPath,
				},
				State::FirstSlash {
					scheme_end,
					authority_or_path_start,
				} => match c {
					'/' => State::SecondSlash {
						scheme_end: Some(scheme_end),
						authority_or_path_start,
					},
					'?' => State::Query {
						scheme_end: Some(scheme_end),
						authority_range: None,
						path_range: authority_or_path_start..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end: Some(scheme_end),
						authority_range: None,
						path_range: authority_or_path_start..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::Path {
						scheme_end: Some(scheme_end),
						authority_range: None,
						path_start: authority_or_path_start,
					},
				},
				State::SecondSlash {
					scheme_end,
					authority_or_path_start,
				} => match c {
					'/' => State::Authority {
						scheme_end,
						authority_start: authority_or_path_start + 2,
					},
					'?' => State::Query {
						scheme_end,
						authority_range: None,
						path_range: authority_or_path_start..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end,
						authority_range: None,
						path_range: authority_or_path_start..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::Path {
						scheme_end,
						authority_range: None,
						path_start: authority_or_path_start,
					},
				},
				State::Authority {
					scheme_end,
					authority_start,
				} => match c {
					'/' => State::Path {
						scheme_end,
						authority_range: Some(authority_start..i),
						path_start: i,
					},
					'?' => State::Query {
						scheme_end,
						authority_range: Some(authority_start..i),
						path_range: i..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end,
						authority_range: Some(authority_start..i),
						path_range: i..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::Authority {
						scheme_end,
						authority_start,
					},
				},
				State::Path {
					scheme_end,
					authority_range,
					path_start,
				} => match c {
					'?' => State::Query {
						scheme_end,
						authority_range,
						path_range: path_start..i,
						query_start: i + 1,
					},
					'#' => State::Fragment {
						scheme_end,
						authority_range,
						path_range: path_start..i,
						query_range: None,
						fragment_start: i + 1,
					},
					_ => State::Path {
						scheme_end,
						authority_range,
						path_start,
					},
				},
				State::Query {
					scheme_end,
					authority_range,
					path_range,
					query_start,
				} => match c {
					'#' => State::Fragment {
						scheme_end,
						authority_range,
						path_range,
						query_range: Some(query_start..i),
						fragment_start: i + 1,
					},
					_ => State::Query {
						scheme_end,
						authority_range,
						path_range,
						query_start,
					},
				},
				fragment => fragment,
			}
		}

		struct Ranges {
			scheme_end: Option<usize>,
			authority: Option<Range<usize>>,
			path: Range<usize>,
			query: Option<Range<usize>>,
			fragment_start: Option<usize>,
		}

		let ranges = match q {
			State::Start => Ranges {
				scheme_end: None,
				authority: None,
				path: 0..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::SchemeOrPath => Ranges {
				scheme_end: None,
				authority: None,
				path: 0..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::FirstSlash {
				scheme_end,
				authority_or_path_start,
			} => Ranges {
				scheme_end: Some(scheme_end),
				authority: None,
				path: authority_or_path_start..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::SecondSlash {
				scheme_end,
				authority_or_path_start,
			} => Ranges {
				scheme_end,
				authority: None,
				path: authority_or_path_start..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::Authority {
				scheme_end,
				authority_start,
			} => Ranges {
				scheme_end,
				authority: Some(authority_start..self.0.len()),
				path: self.0.len()..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::Path {
				scheme_end,
				authority_range,
				path_start,
			} => Ranges {
				scheme_end,
				authority: authority_range,
				path: path_start..self.0.len(),
				query: None,
				fragment_start: None,
			},
			State::Query {
				scheme_end,
				authority_range,
				path_range,
				query_start,
			} => Ranges {
				scheme_end,
				authority: authority_range,
				path: path_range,
				query: Some(query_start..self.0.len()),
				fragment_start: None,
			},
			State::Fragment {
				scheme_end,
				authority_range,
				path_range,
				query_range,
				fragment_start,
			} => Ranges {
				scheme_end,
				authority: authority_range,
				path: path_range,
				query: query_range,
				fragment_start: Some(fragment_start),
			},
		};

		IriRefParts {
			scheme: ranges
				.scheme_end
				.map(|e| unsafe { Scheme::new_unchecked(&self.as_bytes()[..e]) }),
			authority: ranges
				.authority
				.map(|r| unsafe { Authority::new_unchecked(&self.0[r]) }),
			path: unsafe { Path::new_unchecked(&self.0[ranges.path]) },
			query: ranges
				.query
				.map(|r| unsafe { Query::new_unchecked(&self.0[r]) }),
			fragment: ranges
				.fragment_start
				.map(|s| unsafe { Fragment::new_unchecked(&self.0[s..]) }),
		}
	}

	/// Returns the scheme of the IRI reference, if any.
	#[inline]
	pub fn scheme(&self) -> Option<&Scheme> {
		for (i, c) in self.0.char_indices() {
			match c {
				'/' => return None,
				':' => return Some(unsafe { Scheme::new_unchecked(&self.0.as_bytes()[..i]) }),
				_ => (),
			}
		}

		None
	}

	/// Returns the authority part of the IRI reference, if any.
	pub fn authority(&self) -> Option<&Authority> {
		#[derive(Clone, Copy)]
		pub enum State {
			Start,
			SchemeOrPath,
			FirstSlash,
			SecondSlash,
			Capture(usize, usize),
		}

		let mut q = State::Start;
		for (i, c) in self.0.char_indices() {
			q = match q {
				State::Start => match c {
					':' => State::FirstSlash,
					'/' => State::SecondSlash,
					_ => State::SchemeOrPath,
				},
				State::SchemeOrPath => match c {
					':' => State::FirstSlash,
					'/' | '?' | '#' => break,
					_ => State::SchemeOrPath,
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

impl fmt::Display for IriRef {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PARTS: [(
		&str,
		(Option<&str>, Option<&str>, &str, Option<&str>, Option<&str>),
	); 36] = [
		// 0 components.
		("", (None, None, "", None, None)),
		// 1 component.
		("scheme:", (Some("scheme"), None, "", None, None)),
		("//authority", (None, Some("authority"), "", None, None)),
		("path", (None, None, "path", None, None)),
		("/path", (None, None, "/path", None, None)),
		("/", (None, None, "/", None, None)),
		("foo//bar", (None, None, "foo//bar", None, None)),
		("?query", (None, None, "", Some("query"), None)),
		("#fragment", (None, None, "", None, Some("fragment"))),
		(
			"scheme:?query",
			(Some("scheme"), None, "", Some("query"), None),
		),
		// 2 components.
		(
			"scheme://authority",
			(Some("scheme"), Some("authority"), "", None, None),
		),
		("scheme:path", (Some("scheme"), None, "path", None, None)),
		("scheme:/path", (Some("scheme"), None, "/path", None, None)),
		(
			"scheme:?query",
			(Some("scheme"), None, "", Some("query"), None),
		),
		(
			"scheme:#fragment",
			(Some("scheme"), None, "", None, Some("fragment")),
		),
		(
			"//authority/path",
			(None, Some("authority"), "/path", None, None),
		),
		(
			"//authority?query",
			(None, Some("authority"), "", Some("query"), None),
		),
		(
			"//authority#fragment",
			(None, Some("authority"), "", None, Some("fragment")),
		),
		("path?query", (None, None, "path", Some("query"), None)),
		("/path?query", (None, None, "/path", Some("query"), None)),
		(
			"path#fragment",
			(None, None, "path", None, Some("fragment")),
		),
		(
			"?query#fragment",
			(None, None, "", Some("query"), Some("fragment")),
		),
		// 3 components
		(
			"scheme://authority/path",
			(Some("scheme"), Some("authority"), "/path", None, None),
		),
		(
			"scheme://authority?query",
			(Some("scheme"), Some("authority"), "", Some("query"), None),
		),
		(
			"scheme://authority#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"",
				None,
				Some("fragment"),
			),
		),
		(
			"scheme:path?query",
			(Some("scheme"), None, "path", Some("query"), None),
		),
		(
			"scheme:path#fragment",
			(Some("scheme"), None, "path", None, Some("fragment")),
		),
		(
			"//authority/path?query",
			(None, Some("authority"), "/path", Some("query"), None),
		),
		(
			"//authority/path#fragment",
			(None, Some("authority"), "/path", None, Some("fragment")),
		),
		(
			"//authority?query#fragment",
			(None, Some("authority"), "", Some("query"), Some("fragment")),
		),
		(
			"path?query#fragment",
			(None, None, "path", Some("query"), Some("fragment")),
		),
		// 4 components
		(
			"scheme://authority/path?query",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				Some("query"),
				None,
			),
		),
		(
			"scheme://authority/path#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				None,
				Some("fragment"),
			),
		),
		(
			"scheme://authority?query#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"",
				Some("query"),
				Some("fragment"),
			),
		),
		(
			"scheme:path?query#fragment",
			(
				Some("scheme"),
				None,
				"path",
				Some("query"),
				Some("fragment"),
			),
		),
		// 5 components
		(
			"scheme://authority/path?query#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				Some("query"),
				Some("fragment"),
			),
		),
	];

	#[test]
	fn parts() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.scheme.map(Scheme::as_str), expected.0);
			assert_eq!(parts.authority.map(Authority::as_str), expected.1);
			assert_eq!(parts.path.as_str(), expected.2);
			assert_eq!(parts.query.map(Query::as_str), expected.3);
			assert_eq!(parts.fragment.map(Fragment::as_str), expected.4)
		}
	}

	#[test]
	fn scheme() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			assert_eq!(input.scheme().map(Scheme::as_str), expected.0)
		}
	}

	#[test]
	fn authority() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			eprintln!("{input}: {expected:?}");
			assert_eq!(input.authority().map(Authority::as_str), expected.1)
		}
	}
}
