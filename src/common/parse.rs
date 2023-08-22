use std::ops::Range;

fn is_scheme_char(b: u8) -> bool {
	// ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
	b.is_ascii_alphanumeric() | matches!(b, b'+' | b'-' | b'.')
}

/// Checks if the input byte string looks like a scheme.
///
/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
/// valid scheme, of `false` otherwise.
#[inline]
pub fn looks_like_scheme(bytes: &[u8]) -> bool {
	let mut i = 0;
	while i < bytes.len() {
		if i == 0 {
			if !bytes[i].is_ascii_alphabetic() {
				break;
			}
		} else {
			let b = bytes[i];
			if b == b':' {
				return true;
			} else if !is_scheme_char(b) {
				break;
			}
		}

		i += 1
	}

	false
}

#[derive(Debug, PartialEq, Eq)]
pub enum SchemeAuthorityOrPath {
	Scheme,
	Authority,
	Path,
}

pub fn scheme_authority_or_path(bytes: &[u8], mut i: usize) -> (SchemeAuthorityOrPath, usize) {
	pub enum State {
		Start,
		SchemeOrPath,
		Path,
		SecondSlash,
		Authority,
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
					_ => State::SchemeOrPath,
				},
				State::SchemeOrPath => match c {
					b':' => break SchemeAuthorityOrPath::Scheme,
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::SchemeOrPath,
				},
				State::Path => match c {
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::Path,
				},
				State::SecondSlash => match c {
					b'/' => State::Authority,
					b'?' | b'#' => break SchemeAuthorityOrPath::Path,
					_ => State::Path,
				},
				State::Authority => match c {
					b'/' => break SchemeAuthorityOrPath::Authority,
					b'?' | b'#' => break SchemeAuthorityOrPath::Authority,
					_ => State::Authority,
				},
			};

			i += 1
		} else {
			break match q {
				State::Start | State::SchemeOrPath | State::Path | State::SecondSlash => {
					SchemeAuthorityOrPath::Path
				}
				State::Authority => SchemeAuthorityOrPath::Authority,
			};
		}
	};

	(component, i)
}

pub fn scheme(bytes: &[u8], mut i: usize) -> Range<usize> {
	let start = i;

	while i < bytes.len() {
		if bytes[i] == b':' {
			break;
		}

		i += 1
	}

	start..i
}

pub fn find_scheme(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	let start = i;

	while i < bytes.len() {
		match bytes[i] {
			b'/' | b'?' | b'#' => break,
			b':' => return Some(start..i),
			_ => i += 1,
		}
	}

	None
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthorityOrPath {
	Authority,
	Path,
}

pub fn authority_or_path(bytes: &[u8], mut i: usize) -> (AuthorityOrPath, usize) {
	pub enum State {
		Start,
		SecondSlash,
		Path,
		Authority,
	}

	let mut q = State::Start;
	let component = loop {
		if i < bytes.len() {
			let c = bytes[i];
			q = match q {
				State::Start => match c {
					b'?' | b'#' => break AuthorityOrPath::Path,
					b'/' => State::SecondSlash,
					_ => State::Path,
				},
				State::Path => match c {
					b'?' | b'#' => break AuthorityOrPath::Path,
					_ => State::Path,
				},
				State::SecondSlash => match c {
					b'/' => State::Authority,
					b'?' | b'#' => break AuthorityOrPath::Path,
					_ => State::Path,
				},
				State::Authority => match c {
					b'/' => break AuthorityOrPath::Authority,
					b'?' | b'#' => break AuthorityOrPath::Authority,
					_ => State::Authority,
				},
			};

			i += 1
		} else {
			break match q {
				State::Start | State::Path | State::SecondSlash => AuthorityOrPath::Path,
				State::Authority => AuthorityOrPath::Authority,
			};
		}
	};

	(component, i)
}

pub fn find_authority(bytes: &[u8], i: usize) -> Result<Range<usize>, usize> {
	match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => {
			match self::authority_or_path(bytes, scheme_end + 1) {
				(AuthorityOrPath::Authority, end) => Ok((scheme_end + 3)..end),
				(AuthorityOrPath::Path, _) => Err(scheme_end + 1),
			}
		}
		(SchemeAuthorityOrPath::Authority, end) => Ok(2..end),
		(SchemeAuthorityOrPath::Path, _) => Err(0),
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum UserInfoOrHost {
	UserInfo,
	Host,
}

/// Find the user info or host part starting an authority.
///
/// `bytes` must end at the end of the authority.
pub fn user_info_or_host(bytes: &[u8], mut i: usize) -> (UserInfoOrHost, usize) {
	while i < bytes.len() {
		match bytes[i] {
			b'@' => return (UserInfoOrHost::UserInfo, i),
			b':' => {
				// end of the host, or still in the user-info.
				let end = i;

				while i < bytes.len() {
					if bytes[i] == b'@' {
						return (UserInfoOrHost::UserInfo, i);
					}

					i += 1
				}

				return (UserInfoOrHost::Host, end);
			}
			_ => i += 1,
		}
	}

	(UserInfoOrHost::Host, i)
}

/// Find the user info part in an authority.
///
/// `bytes` must end at the end of the authority.
pub fn find_user_info(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	let start = i;
	while i < bytes.len() {
		if bytes[i] == b'@' {
			return Some(start..i);
		}

		i += 1;
	}

	None
}

pub fn host(bytes: &[u8], mut i: usize) -> usize {
	while i < bytes.len() && bytes[i] != b':' {
		i += 1
	}

	i
}

pub fn find_host(bytes: &[u8], i: usize) -> Range<usize> {
	match user_info_or_host(bytes, i) {
		(UserInfoOrHost::UserInfo, i) => {
			let end = host(bytes, i + 1);
			(i + 1)..end
		}
		(UserInfoOrHost::Host, end) => i..end,
	}
}

/// Parse a port starting a the given position.
///
/// `bytes` must end at the end of the authority.
pub fn port(bytes: &[u8], i: usize) -> (bool, usize) {
	if i < bytes.len() && bytes[i] == b':' {
		(true, bytes.len())
	} else {
		(false, i)
	}
}

/// Find the port part in an authority.
///
/// `bytes` must end at the end of the authority.
pub fn find_port(bytes: &[u8], mut i: usize) -> Option<Range<usize>> {
	'host: while i < bytes.len() {
		if bytes[i] == b':' {
			let start = i + 1;

			while i < bytes.len() {
				if bytes[i] == b'@' {
					i += 1;
					continue 'host;
				}

				i += 1;
			}

			return Some(start..i);
		}

		i += 1
	}

	None
}

pub fn path(bytes: &[u8], mut i: usize) -> usize {
	while i < bytes.len() && !matches!(bytes[i], b'?' | b'#') {
		i += 1;
	}

	i
}

pub fn find_path(bytes: &[u8], i: usize) -> Range<usize> {
	match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => {
			match self::authority_or_path(bytes, scheme_end + 1) {
				(AuthorityOrPath::Authority, authority_end) => {
					let end = self::path(bytes, authority_end);
					authority_end..end
				}
				(AuthorityOrPath::Path, end) => (scheme_end + 1)..end,
			}
		}
		(SchemeAuthorityOrPath::Authority, authority_end) => {
			let end = self::path(bytes, authority_end);
			authority_end..end
		}
		(SchemeAuthorityOrPath::Path, end) => 0..end,
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

pub fn find_query(bytes: &[u8], mut i: usize) -> Result<Range<usize>, usize> {
	while i < bytes.len() {
		match bytes[i] {
			b'#' => break,
			b'?' => {
				i += 1;
				let start = i;
				while i < bytes.len() && bytes[i] != b'#' {
					i += 1;
				}

				return Ok(start..i);
			}
			_ => {
				i += 1;
			}
		}
	}

	Err(i)
}

pub fn fragment(bytes: &[u8], i: usize) -> (bool, usize) {
	if i < bytes.len() && bytes[i] == b'#' {
		(true, bytes.len())
	} else {
		(false, bytes.len())
	}
}

pub fn find_fragment(bytes: &[u8], mut i: usize) -> Result<Range<usize>, usize> {
	while i < bytes.len() {
		match bytes[i] {
			b'#' => return Ok((i + 1)..bytes.len()),
			_ => {
				i += 1;
			}
		}
	}

	Err(i)
}

pub struct ReferenceParts {
	pub scheme: Option<Range<usize>>,
	pub authority: Option<Range<usize>>,
	pub path: Range<usize>,
	pub query: Option<Range<usize>>,
	pub fragment: Option<Range<usize>>,
}

pub fn reference_parts(bytes: &[u8], i: usize) -> ReferenceParts {
	let path;
	let (scheme, authority) = match self::scheme_authority_or_path(bytes, i) {
		(SchemeAuthorityOrPath::Scheme, scheme_end) => {
			let authority = match self::authority_or_path(bytes, scheme_end + 1) {
				(AuthorityOrPath::Authority, authority_end) => {
					let path_end = self::path(bytes, authority_end);
					path = authority_end..path_end;
					Some((scheme_end + 3)..authority_end)
				}
				(AuthorityOrPath::Path, path_end) => {
					path = (scheme_end + 1)..path_end;
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
	let query = has_query.then_some((path.end + 1)..query_end);

	let (has_fragment, fragment_end) = self::fragment(bytes, query_end);
	let fragment = has_fragment.then_some((query_end + 1)..fragment_end);

	ReferenceParts {
		scheme,
		authority,
		path,
		query,
		fragment,
	}
}

pub struct Parts {
	pub scheme: Range<usize>,
	pub authority: Option<Range<usize>>,
	pub path: Range<usize>,
	pub query: Option<Range<usize>>,
	pub fragment: Option<Range<usize>>,
}

pub fn parts(bytes: &[u8], i: usize) -> Parts {
	let scheme = self::scheme(bytes, i);

	let path;
	let authority = match self::authority_or_path(bytes, scheme.end + 1) {
		(AuthorityOrPath::Authority, authority_end) => {
			let path_end = self::path(bytes, authority_end);
			path = authority_end..path_end;
			Some((scheme.end + 3)..authority_end)
		}
		(AuthorityOrPath::Path, path_end) => {
			path = (scheme.end + 1)..path_end;
			None
		}
	};

	let (has_query, query_end) = self::query(bytes, path.end);
	let query = has_query.then_some((path.end + 1)..query_end);

	let (has_fragment, fragment_end) = self::fragment(bytes, query_end);
	let fragment = has_fragment.then_some((query_end + 1)..fragment_end);

	Parts {
		scheme,
		authority,
		path,
		query,
		fragment,
	}
}

#[cfg(test)]
mod tests {
	use super::{AuthorityOrPath, SchemeAuthorityOrPath, UserInfoOrHost};

	#[test]
	fn scheme_authority_or_path() {
		let vectors = [
			("http://example.org", (SchemeAuthorityOrPath::Scheme, 4)),
			("//example.org/path", (SchemeAuthorityOrPath::Authority, 13)),
			("", (SchemeAuthorityOrPath::Path, 0)),
			("/", (SchemeAuthorityOrPath::Path, 1)),
			("path", (SchemeAuthorityOrPath::Path, 4)),
			("/path", (SchemeAuthorityOrPath::Path, 5)),
		];

		for (input, expected) in vectors {
			let output = super::scheme_authority_or_path(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn scheme() {
		let vectors = [("http://example.org", 0..4), ("https:foo:bar", 0..5)];

		for (input, expected) in vectors {
			let output = super::scheme(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_scheme() {
		let vectors = [
			("http://example.org", Some(0..4)),
			("https:foo:bar", Some(0..5)),
			("", None),
			("/foo:bar", None),
			("//user:1@host.org/", None),
		];

		for (input, expected) in vectors {
			let output = super::find_scheme(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn authority_or_path() {
		let vectors = [
			("//example.org/path", (AuthorityOrPath::Authority, 13)),
			("", (AuthorityOrPath::Path, 0)),
			("/", (AuthorityOrPath::Path, 1)),
			("path", (AuthorityOrPath::Path, 4)),
			("/path", (AuthorityOrPath::Path, 5)),
		];

		for (input, expected) in vectors {
			let output = super::authority_or_path(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_authority() {
		let vectors = [
			("http://example.org", Ok("example.org")),
			("https:foo:bar", Err(6)),
			("", Err(0)),
			("/foo:bar", Err(0)),
			("//user:1@host.org/", Ok("user:1@host.org")),
		];

		for (input, expected) in vectors {
			let output = super::find_authority(input.as_bytes(), 0).map(|r| &input[r]);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn user_info_or_host() {
		let vectors = [
			("user@example.org", (UserInfoOrHost::UserInfo, 4)),
			("user:12@example.org", (UserInfoOrHost::UserInfo, 7)),
			(":12:user@example.org", (UserInfoOrHost::UserInfo, 8)),
			("example.org", (UserInfoOrHost::Host, 11)),
			("example.org:12", (UserInfoOrHost::Host, 11)),
			(":12", (UserInfoOrHost::Host, 0)),
		];

		for (input, expected) in vectors {
			let output = super::user_info_or_host(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_user_info() {
		let vectors = [
			("example.org:12", None),
			("user@example.org:12", Some("user")),
		];

		for (input, expected) in vectors {
			let output = super::find_user_info(input.as_bytes(), 0).map(|r| &input[r]);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn host() {
		let vectors = [("example.org:12", 11), ("example.org", 11), ("", 0)];

		for (input, expected) in vectors {
			let output = super::host(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_host() {
		let vectors = [
			("example.org:12", "example.org"),
			("user@example.org:12", "example.org"),
			("example.org", "example.org"),
			("user:21@example.org:12", "example.org"),
			("user:32@example.org", "example.org"),
		];

		for (input, expected) in vectors {
			let r = super::find_host(input.as_bytes(), 0);
			let output = &input[r];
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn port() {
		let vectors = [(":12", (true, 3)), ("", (false, 0))];

		for (input, expected) in vectors {
			let output = super::port(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_port() {
		let vectors = [
			("example.org:12", Some("12")),
			("user@example.org:12", Some("12")),
			("example.org", None),
			("user:21@example.org:12", Some("12")),
			("user:32@example.org", None),
		];

		for (input, expected) in vectors {
			let output = super::find_port(input.as_bytes(), 0).map(|r| &input[r]);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn path() {
		let vectors = [
			("foo/bar?query", 7),
			("foo/bar#fragment", 7),
			("foo/bar", 7),
		];

		for (input, expected) in vectors {
			let output = super::path(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_path() {
		let vectors = [
			("scheme:", ""),
			("scheme://example.org/", "/"),
			("foo/bar?query", "foo/bar"),
			("foo/bar#fragment", "foo/bar"),
			("foo/bar", "foo/bar"),
		];

		for (input, expected) in vectors {
			let r = super::find_path(input.as_bytes(), 0);
			let output = &input[r];
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn query() {
		let vectors = [
			("", (false, 0)),
			("?query", (true, 6)),
			("?query#fragment", (true, 6)),
		];

		for (input, expected) in vectors {
			let output = super::query(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_query() {
		let vectors = [
			("path?query", Ok(5..10)),
			("path?query#fragment", Ok(5..10)),
			("path#fragment", Err(4)),
		];

		for (input, expected) in vectors {
			let output = super::find_query(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn fragment() {
		let vectors = [("", (false, 0)), ("#fragment", (true, 9))];

		for (input, expected) in vectors {
			let output = super::fragment(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}

	#[test]
	fn find_fragment() {
		let vectors = [
			("path?query", Err(10)),
			("path?query#fragment", Ok(11..19)),
			("path#fragment", Ok(5..13)),
		];

		for (input, expected) in vectors {
			let output = super::find_fragment(input.as_bytes(), 0);
			assert_eq!(output, expected)
		}
	}
}
