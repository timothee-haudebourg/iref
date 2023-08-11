use std::ops::Range;

pub enum SchemeAuthorityOrPath {
	Scheme,
	Authority,
	Path
}

pub fn scheme_authority_or_path(bytes: &[u8], mut i: usize) -> (SchemeAuthorityOrPath, usize) {
	pub enum State {
		Start,
		SchemeOrPath,
		Path,
		SecondSlash,
		Authority
	}

	let mut q = State::Start;
	let component = loop {
		if i < bytes.len() {
			let c = bytes[i];
			q = match q {
				State::Start => match c {
					b':' => break SchemeAuthorityOrPath::Scheme,
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					b'/' => State::SecondSlash,
					_ => State::SchemeOrPath
				}
				State::SchemeOrPath => match c {
					b':' => break SchemeAuthorityOrPath::Scheme,
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::SchemeOrPath
				}
				State::Path => match c {
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::Path
				}
				State::SecondSlash => match c {
					b'/' => State::Authority,
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::Path
				}
				State::Authority => match c {
					b'/' => break SchemeAuthorityOrPath::Authority,
					b'?' | b'#' => break SchemeAuthorityOrPath::Authority,
					_ => State::Authority
				}
			};

			i += 1
		} else {
			break match q {
				State::Start | State::SchemeOrPath | State::Path | State::SecondSlash => SchemeAuthorityOrPath::Path,
				State::Authority => SchemeAuthorityOrPath::Authority
			}
		}
	};

	(component, i)
}

pub fn scheme(bytes: &[u8], mut i: usize) -> Range<usize> {
	let start = i;

	while i < bytes.len() {
		if bytes[i] == b':' {
			break
		}
	}

	start..i
}

pub fn find_scheme(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	let start = i;

	while i < bytes.len() {
		match bytes[i] {
			b'/' | b'?' | b'#' => break,
			b':' => return Some(start..i),
			_ => i += 1
		}
	}

	None
}

pub enum AuthorityOrPath {
	Authority,
	Path
}

pub fn authority_or_path(bytes: &[u8], mut i: usize) -> (AuthorityOrPath, usize) {
	pub enum State {
		Start,
		SecondSlash,
		Path,
		Authority
	}

	let mut q = State::Start;
	let component = loop {
		if i < bytes.len() {
			let c = bytes[i];
			q = match q {
				State::Start => match c {
					b'?' | b'#' => break AuthorityOrPath::Path,
					b'/' => State::SecondSlash,
					_ => State::Path
				}
				State::Path => match c {
					b'?' | b'#' => break AuthorityOrPath::Path,
					_ => State::Path
				}
				State::SecondSlash => match c {
					b'/' => State::Authority,
					b'?' | b'#' => break AuthorityOrPath::Path,
					_ => State::Path
				}
				State::Authority => match c {
					b'/' => break AuthorityOrPath::Authority,
					b'?' | b'#' => break AuthorityOrPath::Authority,
					_ => State::Authority
				}
			};

			i += 1
		} else {
			break match q {
				State::Start | State::Path | State::SecondSlash => AuthorityOrPath::Path,
				State::Authority => AuthorityOrPath::Authority
			}
		}
	};

	(component, i)
}

pub fn find_authority(bytes: &[u8], i: usize) -> Option<Range<usize>> {
	match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => match self::authority_or_path(bytes, scheme_end+1) {
			(AuthorityOrPath::Authority, end) => Some((scheme_end+3)..end),
			(AuthorityOrPath::Path, _) => None
		}
		(SchemeAuthorityOrPath::Authority, end) => Some(2..end),
		(SchemeAuthorityOrPath::Path, _) => None
	}
}

pub fn path(bytes: &[u8], mut i: usize) -> usize {
	while i < bytes.len() && !matches!(bytes[i], b'?' | b'#') {
		i += 1;
	}
	
	i
}

pub fn find_path(bytes: &[u8], i: usize) -> Range<usize> {
	match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => match self::authority_or_path(bytes, scheme_end+1) {
			(AuthorityOrPath::Authority, authority_end) => {
				let end = self::path(bytes, authority_end);
				authority_end..end
			}
			(AuthorityOrPath::Path, end) => {
				(scheme_end+1)..end
			}
		}
		(SchemeAuthorityOrPath::Authority, authority_end) => {
			let end = self::path(bytes, authority_end);
			authority_end..end
		},
		(SchemeAuthorityOrPath::Path, end) => {
			0..end
		}
	}
}

pub fn query(bytes: &[u8], mut i: usize) -> (bool, usize) {
	if i < bytes.len() && bytes[i] == b'?' {
		i += 1;
		while i < bytes.len() && bytes[i] != b'#' {
			i += 1;
		}

		(true, i)
	} else {
		(false, i)
	}
}

pub fn find_query(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	while i < bytes.len() {
		match bytes[i] {
			b'#' => break,
			b'?' => {
				i += 1;
				let start = i;
				while i < bytes.len() && bytes[i] != b'#' {
					i += 1;
				}

				return Some(start..i)
			}
			_ => {
				i += 1;
			}
		}
	}

	None
}

pub fn fragment(bytes: &[u8], i: usize) -> (bool, usize) {
	if i < bytes.len() && bytes[i] == b'#' {
		(true, bytes.len())
	} else {
		(false, bytes.len())
	}
}

pub fn find_fragment(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	while i < bytes.len() {
		match bytes[i] {
			b'#' => {
				return Some((i+1)..bytes.len())
			}
			_ => {
				i += 1;
			}
		}
	}

	None
}

pub struct ReferenceParts {
	pub scheme: Option<Range<usize>>,
	pub authority: Option<Range<usize>>,
	pub path: Range<usize>,
	pub query: Option<Range<usize>>,
	pub fragment: Option<Range<usize>>
}

pub fn reference_parts(bytes: &[u8], i: usize) -> ReferenceParts {
	let path;
	let (scheme, authority) = match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => {
			let authority = match self::authority_or_path(bytes, scheme_end + 1) {
				(AuthorityOrPath::Authority, authority_end) => {
					let path_end = self::path(bytes, authority_end);
					path = authority_end..path_end;
					Some((scheme_end+3)..authority_end)
				}
				(AuthorityOrPath::Path, path_end) => {
					path = (scheme_end+1)..path_end;
					None
				}
			};

			(Some(0..scheme_end), authority)
		}
		(SchemeAuthorityOrPath::Authority, authority_end) => {
			let path_end = self::path(bytes, authority_end);
			path = authority_end..path_end;
			(None, Some(2..authority_end))
		}
		(SchemeAuthorityOrPath::Path, path_end) => {
			path = 0..path_end;
			(None, None)
		}
	};

	let (has_query, query_end) = self::query(bytes, path.end);
	let query = has_query.then_some((path.end+1)..query_end);

	let (has_fragment, fragment_end) = self::fragment(bytes, query_end);
	let fragment = has_fragment.then_some((query_end+1)..fragment_end);

	ReferenceParts { scheme, authority, path, query, fragment }
}

pub struct Parts {
	pub scheme: Range<usize>,
	pub authority: Option<Range<usize>>,
	pub path: Range<usize>,
	pub query: Option<Range<usize>>,
	pub fragment: Option<Range<usize>>
}

pub fn parts(bytes: &[u8], i: usize) -> Parts {
	let scheme = self::scheme(bytes, i);

	let path;
	let authority = match self::authority_or_path(bytes, scheme.end + 1) {
		(AuthorityOrPath::Authority, authority_end) => {
			let path_end = self::path(bytes, authority_end);
			path = authority_end..path_end;
			Some((scheme.end+3)..authority_end)
		}
		(AuthorityOrPath::Path, path_end) => {
			path = (scheme.end+1)..path_end;
			None
		}
	};

	let (has_query, query_end) = self::query(bytes, path.end);
	let query = has_query.then_some((path.end+1)..query_end);

	let (has_fragment, fragment_end) = self::fragment(bytes, query_end);
	let fragment = has_fragment.then_some((query_end+1)..fragment_end);

	Parts { scheme, authority, path, query, fragment }
}