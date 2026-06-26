use core::{
	cmp::Ordering,
	hash::{self, Hash},
	ops::Deref,
};
use static_automata::grammar;

pub use crate::{InvalidScheme, Scheme};

#[cfg(feature = "std")]
pub use crate::SchemeBuf;

mod authority;
mod error;
mod fragment;
mod path;
mod query;
mod reference;

pub use authority::*;
pub use error::*;
pub use fragment::*;
pub use path::*;
pub use query::*;
pub use reference::*;

#[grammar(
	file = "grammar.abnf",
	export("URI", "URI-reference" as UriRef, "authority", "host", "userinfo" as UserInfo, "path", "segment", "query", "fragment")
)]
pub(crate) mod grammar {}

/// Uniform Resource Identifier (URI).
///
/// # Example
///
/// ```rust
/// use iref::uri::{Uri, Scheme, Authority, Path, Query, Fragment};
/// # fn main() -> Result<(), iref::InvalidUri<&'static str>> {
/// let uri = Uri::new("https://www.rust-lang.org/foo/bar?query#fragment")?;
///
/// assert_eq!(uri.scheme(), Scheme::new(b"https").unwrap());
/// assert_eq!(uri.authority(), Some(Authority::new("www.rust-lang.org").unwrap()));
/// assert_eq!(uri.path(), Path::new("/foo/bar").unwrap());
/// assert_eq!(uri.query(), Some(Query::new("query").unwrap()));
/// assert_eq!(uri.fragment(), Some(Fragment::new("fragment").unwrap()));
/// #
/// # Ok(())
/// # }
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(grammar::Uri)]
#[newtype(name = "URI", no_deref, ord([u8], &[u8], str, &str))]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String), owned(UriBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Uri(str);

impl Uri {
	/// Returns all the parts of this URI.
	///
	/// This method parses the URI and returns a [`UriParts`] struct containing
	/// references to each component: scheme, authority, path, query, and
	/// fragment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::Uri;
	///
	/// let uri = Uri::new("https://example.org:8080/path?query#fragment").unwrap();
	/// let parts = uri.parts();
	///
	/// assert_eq!(parts.scheme, "https");
	/// assert_eq!(parts.authority.unwrap(), "example.org:8080");
	/// assert_eq!(parts.path, "/path");
	/// assert_eq!(parts.query.unwrap(), "query");
	/// assert_eq!(parts.fragment.unwrap(), "fragment");
	/// ```
	pub fn parts(&self) -> UriParts<'_> {
		let bytes = self.as_bytes();
		let ranges = crate::common::parse::parts(bytes, 0);

		UriParts {
			scheme: unsafe { Scheme::new_unchecked_from_bytes(&bytes[ranges.scheme]) },
			authority: ranges
				.authority
				.map(|r| unsafe { Authority::new_unchecked(&self.0[r]) }),
			path: unsafe { Path::new_unchecked(&self.0[ranges.path]) },
			query: ranges
				.query
				.map(|r| unsafe { Query::new_unchecked(&self.0[r]) }),
			fragment: ranges
				.fragment
				.map(|r| unsafe { Fragment::new_unchecked(&self.0[r]) }),
		}
	}

	/// Returns the scheme of the IRI.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::Uri;
	///
	/// let uri = Uri::new("https://example.org/path").unwrap();
	/// assert_eq!(uri.scheme(), "https");
	/// ```
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		let bytes = self.as_bytes();
		let range = crate::common::parse::scheme(bytes, 0);
		unsafe { Scheme::new_unchecked_from_bytes(&bytes[range]) }
	}

	/// The URI without the file name, query and fragment.
	///
	/// # Example
	/// ```
	/// # use iref::Uri;
	/// let a = Uri::new("https://crates.io/crates/iref?query#fragment").unwrap();
	/// let b = Uri::new("https://crates.io/crates/iref/?query#fragment").unwrap();
	/// assert_eq!(a.base(), "https://crates.io/crates/");
	/// assert_eq!(b.base(), "https://crates.io/crates/iref/")
	/// ```
	#[inline]
	pub fn base(&self) -> &Self {
		let bytes = self.as_bytes();
		let path_range = crate::common::parse::find_path(bytes, 0);
		let path_start = path_range.start;
		let path = unsafe { Path::new_unchecked_from_bytes(&bytes[path_range]) };

		let directory_path = path.directory();
		let end = path_start + directory_path.len();
		unsafe { Self::new_unchecked_from_bytes(&bytes[..end]) }
	}

	/// Joins a URI reference to this URI, returning a new owned URI.
	///
	/// This method resolves the given URI reference against this URI as a base,
	/// following the reference resolution algorithm defined in RFC 3986.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, UriRef};
	///
	/// let base = Uri::new("https://example.org/foo/bar").unwrap();
	/// let reference = UriRef::new("../baz").unwrap();
	///
	/// assert_eq!(base.joined(reference), "https://example.org/baz");
	/// ```
	#[cfg(feature = "std")]
	pub fn joined(&self, input: impl AsRef<UriRef>) -> UriBuf {
		let mut result = self.to_owned();
		result.join(input);
		result
	}

	/// Tries to join a string as a URI reference to this URI.
	///
	/// Same as [`Self::joined`] but accepts a `&str` instead of a [`&UriRef`](UriRef).
	/// Returns an error if the input string is not a valid URI reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::Uri;
	///
	/// let base = Uri::new("https://example.org/foo/bar").unwrap();
	///
	/// assert_eq!(base.try_joined("../baz").unwrap(), "https://example.org/baz");
	/// assert!(base.try_joined("not a valid uri ref\0").is_err());
	/// ```
	#[cfg(feature = "std")]
	pub fn try_joined<'r>(&self, input: &'r str) -> Result<UriBuf, InvalidUriRef<&'r str>> {
		UriRef::new(input).map(|r| self.joined(r))
	}

	/// Converts this URI into an URI reference.
	///
	/// All IRI are valid URI references.
	pub fn as_uri_ref(&self) -> &UriRef {
		unsafe { UriRef::new_unchecked(&self.0) }
	}

	/// Returns a copy of this URI with the given scheme.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, Scheme};
	///
	/// let uri = Uri::new("http://example.org/path").unwrap();
	/// let new = uri.with_scheme(Scheme::new(b"https").unwrap());
	/// assert_eq!(new, "https://example.org/path");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_scheme(&self, scheme: &Scheme) -> UriBuf {
		let mut buf = self.to_owned();
		buf.set_scheme(scheme);
		buf
	}

	/// Returns a copy of this URI with the given authority.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, uri::Authority};
	///
	/// let uri = Uri::new("https://example.org/path").unwrap();
	/// let new = uri.with_authority(Some(Authority::new("other.com").unwrap()));
	/// assert_eq!(new, "https://other.com/path");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_authority(&self, authority: Option<&Authority>) -> UriBuf {
		let mut buf = self.to_owned();
		buf.set_authority(authority);
		buf
	}

	/// Returns a copy of this URI with the given path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, uri::Path};
	///
	/// let uri = Uri::new("https://example.org/old").unwrap();
	/// let new = uri.with_path(Path::new("/new").unwrap());
	/// assert_eq!(new, "https://example.org/new");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_path(&self, path: &Path) -> UriBuf {
		let mut buf = self.to_owned();
		buf.set_path(path);
		buf
	}

	/// Returns a copy of this URI with the given query.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, uri::Query};
	///
	/// let uri = Uri::new("https://example.org/path").unwrap();
	/// let new = uri.with_query(Some(Query::new("key=value").unwrap()));
	/// assert_eq!(new, "https://example.org/path?key=value");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_query(&self, query: Option<&Query>) -> UriBuf {
		let mut buf = self.to_owned();
		buf.set_query(query);
		buf
	}

	/// Returns a copy of this URI with the given fragment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{Uri, uri::Fragment};
	///
	/// let uri = Uri::new("https://example.org/path").unwrap();
	/// let new = uri.with_fragment(Some(Fragment::new("section").unwrap()));
	/// assert_eq!(new, "https://example.org/path#section");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_fragment(&self, fragment: Option<&Fragment>) -> UriBuf {
		let mut buf = self.to_owned();
		buf.set_fragment(fragment);
		buf
	}
}

impl Deref for Uri {
	type Target = UriRef;

	fn deref(&self) -> &Self::Target {
		self.as_uri_ref()
	}
}

impl PartialEq for Uri {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a Uri> for Uri {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl Eq for Uri {}

impl PartialOrd for Uri {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a Uri> for Uri {
	fn partial_cmp(&self, other: &&'a Self) -> Option<Ordering> {
		self.partial_cmp(*other)
	}
}

impl Ord for Uri {
	fn cmp(&self, other: &Self) -> Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for Uri {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl PartialEq<UriRef> for Uri {
	fn eq(&self, other: &UriRef) -> bool {
		*self.as_uri_ref() == *other
	}
}

impl PartialOrd<UriRef> for Uri {
	fn partial_cmp(&self, other: &UriRef) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other)
	}
}

#[cfg(feature = "std")]
impl PartialEq<UriRefBuf> for Uri {
	fn eq(&self, other: &UriRefBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialOrd<UriRefBuf> for Uri {
	fn partial_cmp(&self, other: &UriRefBuf) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

/// Individual components of a URI.
///
/// Contains references to each component of a URI as defined
/// in RFC 3986: scheme, authority, path, query, and fragment.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriParts<'a> {
	/// Scheme component (e.g., `https`, `http`, `file`).
	pub scheme: &'a Scheme,

	/// Authority component, if present.
	///
	/// Contains the host and optionally userinfo and port
	/// (e.g., `user@example.org:8080`).
	pub authority: Option<&'a Authority>,

	/// Path component.
	///
	/// May be empty, but is always present.
	pub path: &'a Path,

	/// Query component, if present.
	pub query: Option<&'a Query>,

	/// Fragment component, if present.
	pub fragment: Option<&'a Fragment>,
}

#[cfg(feature = "std")]
impl UriBuf {
	#[inline]
	unsafe fn replace(&mut self, range: core::ops::Range<usize>, content: &[u8]) {
		crate::utils::replace(unsafe { self.as_mut_vec() }, range, content)
	}

	#[inline]
	unsafe fn allocate(&mut self, range: core::ops::Range<usize>, len: usize) {
		crate::utils::allocate_range(unsafe { self.as_mut_vec() }, range, len)
	}

	/// Returns a mutable reference to the authority part, if present.
	///
	/// The returned [`AuthorityMut`] allows in-place modification of the
	/// authority component (host, port, userinfo).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriBuf;
	///
	/// let mut uri = UriBuf::new("https://example.org:8080/path".to_string()).unwrap();
	///
	/// if let Some(mut authority) = uri.authority_mut() {
	///     authority.set_host("other.com".try_into().unwrap());
	/// }
	///
	/// assert_eq!(uri, "https://other.com:8080/path");
	/// ```
	#[inline]
	pub fn authority_mut(&mut self) -> Option<AuthorityMut<'_>> {
		crate::common::parse::find_authority(self.as_bytes(), 0)
			.ok()
			.map(|range| unsafe { AuthorityMut::new_unchecked(self.as_mut_vec(), range) })
	}

	/// Sets the authority part.
	///
	/// If the path is relative, this also turns it into an absolute path,
	/// since an authority cannot be followed by a relative path.
	///
	/// To avoid any ambiguity, if `authority` is `None` and the path starts
	/// with `//`, it will be changed into `/.//` as to not be interpreted as
	/// an authority part.
	///
	/// # Example
	///
	/// ```
	/// use iref::{UriRefBuf, uri::Authority};
	///
	/// let mut a = UriRefBuf::new("scheme:/path".to_string()).unwrap();
	/// a.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(a, "scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = UriRefBuf::new("scheme:path".to_string()).unwrap();
	/// b.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(b, "scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = UriRefBuf::new("scheme://example.org//path".to_string()).unwrap();
	/// c.set_authority(None);
	/// assert_eq!(c, "scheme:/.//path");
	/// ```
	#[inline]
	pub fn set_authority(&mut self, authority: Option<&Authority>) {
		let bytes = self.as_bytes();
		match authority {
			Some(new_authority) => match crate::common::parse::find_authority(bytes, 0) {
				Ok(range) => unsafe { self.replace(range, new_authority.as_bytes()) },
				Err(start) => {
					if !bytes[start..].starts_with(b"/") {
						// VALIDITY: When an authority is present, the path must
						//           be absolute.
						unsafe {
							self.allocate(start..start, new_authority.len() + 3);
							let bytes = self.as_mut_vec();
							let delim_end = start + 2;
							bytes[start..delim_end].copy_from_slice(b"//");
							bytes[delim_end..(delim_end + new_authority.len())]
								.copy_from_slice(new_authority.as_bytes());
							bytes[delim_end + new_authority.len()] = b'/';
						}
					} else {
						unsafe {
							self.allocate(start..start, new_authority.len() + 2);
							let bytes = self.as_mut_vec();
							let delim_end = start + 2;
							bytes[start..delim_end].copy_from_slice(b"//");
							bytes[delim_end..(delim_end + new_authority.len())]
								.copy_from_slice(new_authority.as_bytes())
						}
					}
				}
			},
			None => {
				if let Ok(range) = crate::common::parse::find_authority(bytes, 0) {
					let value: &[u8] = if bytes[range.end..].starts_with(b"//") {
						// AMBIGUITY: The URI `http://example.com//foo` would
						//            become `http://foo`, but `//foo` is not
						//            the authority.
						// SOLUTION:  We change `//foo` to `/.//foo`.
						b"/."
					} else {
						b""
					};

					unsafe {
						self.replace((range.start - 2)..range.end, value);
					}
				}
			}
		}
	}

	/// Tries to set the authority.
	///
	/// Same as [`Self::set_authority`] but accepts a `&str` instead of
	/// an [`&Authority`](Authority).
	pub fn try_set_authority<'s>(
		&mut self,
		authority: Option<&'s str>,
	) -> Result<(), InvalidAuthority<&'s str>> {
		self.set_authority(authority.map(TryInto::try_into).transpose()?);
		Ok(())
	}

	/// Returns a mutable reference to the path part.
	///
	/// The returned [`PathMut`] allows in-place modification of the path,
	/// including appending segments, normalizing, and replacing the entire path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriBuf, uri::Segment};
	///
	/// let mut uri = UriBuf::new("https://example.org/foo/../bar?query#frag".to_string()).unwrap();
	/// uri.path_mut().normalize();
	/// assert_eq!(uri, "https://example.org/bar?query#frag");
	///
	/// uri.path_mut().push(Segment::new("baz").unwrap());
	/// assert_eq!(uri, "https://example.org/bar/baz?query#frag");
	/// ```
	#[inline]
	pub fn path_mut(&mut self) -> PathMut<'_> {
		let range = crate::common::parse::find_path(self.as_bytes(), 0);
		unsafe { PathMut::new_unchecked(self.as_mut_vec(), range) }
	}

	/// Sets the path part.
	///
	/// If there is an authority and the path is relative, this also turns it
	/// into an absolute path, since an authority cannot be followed by a
	/// relative path.
	///
	/// To avoid any ambiguity, if there is no authority and the path starts
	/// with `//`, it will be changed into `/.//` as to not be interpreted as
	/// an authority part. Similarly if there is no scheme nor authority and the
	/// beginning of the new path looks like a scheme, it is prefixed with `./`
	/// to not be confused with a scheme.
	///
	/// # Example
	///
	/// ```
	/// use iref::{UriRefBuf, uri::Path};
	///
	/// let mut a = UriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// a.set_path(Path::new("/foo/bar").unwrap());
	/// assert_eq!(a, "http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = UriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// b.set_path(Path::new("relative/path").unwrap());
	/// assert_eq!(b, "http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = UriRefBuf::new("http:old/path".to_string()).unwrap();
	/// c.set_path(Path::new("//foo/bar").unwrap());
	/// assert_eq!(c, "http:/.//foo/bar");
	///
	/// // If there is no authority nor scheme, and the path beginning looks
	/// // like a scheme, it is prefixed with `./` to avoid being confused with
	/// // a scheme.
	/// let mut d = UriRefBuf::new("old/path".to_string()).unwrap();
	/// d.set_path(Path::new("foo:bar").unwrap());
	/// assert_eq!(d, "./foo:bar");
	/// ```
	#[inline]
	pub fn set_path(&mut self, path: &Path) {
		self.path_mut().replace(path);
	}

	/// Tries to set the path.
	///
	/// Same as [`Self::set_path`] but accepts a `&str` instead of
	/// an [`&Path`](Path).
	pub fn try_set_path<'s>(&mut self, path: &'s str) -> Result<(), InvalidPath<&'s str>> {
		self.set_path(path.try_into()?);
		Ok(())
	}

	/// Sets and normalizes the path.
	pub fn set_and_normalize_path(&mut self, path: &Path) {
		self.set_path(path);
		self.path_mut().normalize();
	}

	/// Sets the query part.
	///
	/// If `query` is `Some`, the query component is set to the given value.
	/// If `query` is `None`, the query component is removed entirely.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriBuf, uri::Query};
	///
	/// let mut uri = UriBuf::new("https://example.org/path".to_string()).unwrap();
	///
	/// uri.set_query(Some(Query::new("key=value").unwrap()));
	/// assert_eq!(uri, "https://example.org/path?key=value");
	///
	/// uri.set_query(None);
	/// assert_eq!(uri, "https://example.org/path");
	/// ```
	#[inline]
	pub fn set_query(&mut self, query: Option<&Query>) {
		match query {
			Some(new_query) => match crate::common::parse::find_query(self.as_bytes(), 0) {
				Ok(range) => unsafe { self.replace(range, new_query.as_bytes()) },
				Err(start) => unsafe {
					let new_query = new_query.as_bytes();
					self.allocate(start..start, new_query.len() + 1);
					let bytes = self.as_mut_vec();
					let delim_end = start + 1;
					bytes[start] = b'?';
					bytes[delim_end..(delim_end + new_query.len())].copy_from_slice(new_query)
				},
			},
			None => {
				if let Ok(range) = crate::common::parse::find_query(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start - 1)..range.end, b"");
					}
				}
			}
		}
	}

	/// Tries to set the query part.
	///
	/// Same as [`Self::set_query`] but accepts a `&str` instead of
	/// an [`&Query`](Query).
	pub fn try_set_query<'s>(
		&mut self,
		query: Option<&'s str>,
	) -> Result<(), InvalidQuery<&'s str>> {
		self.set_query(query.map(TryInto::try_into).transpose()?);
		Ok(())
	}

	/// Sets the fragment part.
	///
	/// If `fragment` is `Some`, the fragment component is set to the given value.
	/// If `fragment` is `None`, the fragment component is removed entirely.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriBuf, uri::Fragment};
	///
	/// let mut uri = UriBuf::new("https://example.org/path".to_string()).unwrap();
	///
	/// uri.set_fragment(Some(Fragment::new("section").unwrap()));
	/// assert_eq!(uri, "https://example.org/path#section");
	///
	/// uri.set_fragment(None);
	/// assert_eq!(uri, "https://example.org/path");
	/// ```
	#[inline]
	pub fn set_fragment(&mut self, fragment: Option<&Fragment>) {
		match fragment {
			Some(new_fragment) => match crate::common::parse::find_fragment(self.as_bytes(), 0) {
				Ok(range) => unsafe { self.replace(range, new_fragment.as_bytes()) },
				Err(start) => unsafe {
					let new_fragment = new_fragment.as_bytes();
					self.allocate(start..start, new_fragment.len() + 1);
					let bytes = self.as_mut_vec();
					let delim_end = start + 1;
					bytes[start] = b'#';
					bytes[delim_end..(delim_end + new_fragment.len())].copy_from_slice(new_fragment)
				},
			},
			None => {
				if let Ok(range) = crate::common::parse::find_fragment(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start - 1)..range.end, b"");
					}
				}
			}
		}
	}

	/// Tries to set the fragment part.
	///
	/// Same as [`Self::set_fragment`] but accepts a `&str` instead of
	/// an [`&Fragment`](Fragment).
	pub fn try_set_fragment<'s>(
		&mut self,
		fragment: Option<&'s str>,
	) -> Result<(), InvalidFragment<&'s str>> {
		self.set_fragment(fragment.map(TryInto::try_into).transpose()?);
		Ok(())
	}

	/// Creates a new URI from a scheme.
	///
	/// The resulting URI contains only the scheme followed by a colon,
	/// with no authority, path, query, or fragment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriBuf, SchemeBuf};
	///
	/// let scheme = SchemeBuf::new(b"https".to_vec()).unwrap();
	/// let uri = UriBuf::from_scheme(scheme);
	///
	/// assert_eq!(uri, "https:");
	/// ```
	#[inline]
	pub fn from_scheme(scheme: SchemeBuf) -> Self {
		let mut bytes = scheme.into_bytes();
		bytes.push(b':');
		unsafe { Self::new_unchecked(bytes) }
	}

	/// Sets the scheme part.
	///
	/// # Example
	///
	/// ```
	/// use iref::{UriBuf, Scheme};
	///
	/// let mut a = UriBuf::new("http://example.org/path".to_string()).unwrap();
	/// a.set_scheme(Scheme::new(b"https").unwrap());
	/// assert_eq!(a, "https://example.org/path");
	/// ```
	#[inline]
	pub fn set_scheme(&mut self, new_scheme: &Scheme) {
		let range = crate::common::parse::scheme(self.as_bytes(), 0);
		unsafe { self.replace(range, new_scheme.as_bytes()) }
	}

	/// Joins the given URI reference to this absolute URI in place.
	///
	/// This resolves the given URI reference against this URI as a base,
	/// modifying `self` to contain the result. This is similar to
	/// [`UriRefBuf::resolve`], but with the subject and object swapped.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriBuf, UriRef};
	///
	/// let mut uri = UriBuf::new("https://example.org/foo/bar".to_string()).unwrap();
	/// uri.join(UriRef::new("../baz?query").unwrap());
	///
	/// assert_eq!(uri, "https://example.org/baz?query");
	/// ```
	///
	/// ## Abnormal use of dot segments
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn join(&mut self, input: impl AsRef<UriRef>) {
		let input = input.as_ref();
		let parts = input.parts();

		match parts.scheme {
			Some(scheme) => {
				self.set_scheme(scheme);
				self.set_authority(parts.authority);
				self.set_and_normalize_path(parts.path);
				self.set_query(parts.query);
				self.set_fragment(parts.fragment);
			}
			None => match parts.authority {
				Some(authority) => {
					self.set_authority(Some(authority));
					self.set_and_normalize_path(parts.path);
					self.set_query(parts.query);
					self.set_fragment(parts.fragment);
				}
				None => {
					if parts.path.is_relative() && parts.path.is_empty() {
						if let Some(query) = parts.query {
							self.set_query(Some(query))
						}
					} else if parts.path.is_absolute() {
						self.set_query(parts.query);
						self.set_and_normalize_path(parts.path);
					} else {
						self.set_query(parts.query);
						let has_authority = self.authority().is_some();
						let mut path = self.path_mut();

						if path.is_empty() && has_authority {
							path.replace(Path::EMPTY_ABSOLUTE);
						} else {
							path.pop();
						}

						path.lazy_append(parts.path);
						path.normalize();
					}

					self.set_fragment(parts.fragment);
				}
			},
		}
	}

	/// Tries to join a string as a URI reference to this absolute URI.
	///
	/// Same as [`Self::join`] but accepts a `&str` instead of a [`&UriRef`](UriRef).
	/// Returns an error if the input string is not a valid URI reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriBuf;
	///
	/// let mut uri = UriBuf::new("https://example.org/foo/bar".to_string()).unwrap();
	/// uri.try_join("../baz").unwrap();
	///
	/// assert_eq!(uri, "https://example.org/baz");
	/// ```
	pub fn try_join<'r>(
		&mut self,
		input: &'r str,
	) -> Result<(), <&'r UriRef as TryFrom<&'r str>>::Error> {
		self.join(UriRef::new(input)?);
		Ok(())
	}
}

/// Parses an [`Uri`] at compile time.
#[macro_export]
macro_rules! uri {
	($value:literal) => {
		match $crate::uri::Uri::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI"),
		}
	};
}

#[cfg(feature = "std")]
impl UriBuf {
	/// Converts this URI into a URI reference.
	///
	/// Since all URIs are valid URI references, this conversion is infallible.
	pub fn into_uri_ref(self) -> UriRefBuf {
		unsafe { UriRefBuf::new_unchecked(self.0) }
	}

	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the URI.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid URI.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		unsafe { self.0.as_mut_vec() }
	}
}

#[cfg(feature = "std")]
impl AsRef<UriRef> for UriBuf {
	fn as_ref(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl std::borrow::Borrow<UriRef> for UriBuf {
	fn borrow(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl From<UriBuf> for UriRefBuf {
	fn from(value: UriBuf) -> Self {
		value.into_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialEq<UriRef> for UriBuf {
	fn eq(&self, other: &UriRef) -> bool {
		*self.as_uri_ref() == *other
	}
}

#[cfg(feature = "std")]
impl<'a> PartialEq<&'a UriRef> for UriBuf {
	fn eq(&self, other: &&'a UriRef) -> bool {
		*self.as_uri_ref() == **other
	}
}

#[cfg(feature = "std")]
impl PartialEq<UriRefBuf> for UriBuf {
	fn eq(&self, other: &UriRefBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialOrd<UriRef> for UriBuf {
	fn partial_cmp(&self, other: &UriRef) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other)
	}
}

#[cfg(feature = "std")]
impl<'a> PartialOrd<&'a UriRef> for UriBuf {
	fn partial_cmp(&self, other: &&'a UriRef) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(*other)
	}
}

#[cfg(feature = "std")]
impl PartialOrd<UriRefBuf> for UriBuf {
	fn partial_cmp(&self, other: &UriRefBuf) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn scheme() {
		let vectors: &[(&str, &str)] = &[
			("http:", "http"),
			("https://example.org", "https"),
			("ftp://ftp.example.org/file", "ftp"),
			("mailto:user@example.org", "mailto"),
			("urn:isbn:0451450523", "urn"),
			("file:///path/to/file", "file"),
			("custom-scheme://host", "custom-scheme"),
			("a://b", "a"),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(uri.scheme().as_str(), *expected, "input: {input}");
		}
	}

	#[test]
	fn authority() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", Some("example.org")),
			("http://example.org:8080", Some("example.org:8080")),
			("http://user@example.org", Some("user@example.org")),
			(
				"http://user:pass@example.org:8080",
				Some("user:pass@example.org:8080"),
			),
			("http://[::1]", Some("[::1]")),
			("http://[::1]:8080", Some("[::1]:8080")),
			("mailto:user@example.org", None),
			("urn:isbn:0451450523", None),
			("file:///path/to/file", Some("")),
			("http://", Some("")),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(
				uri.authority().map(|a| a.as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn authority_host() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", Some("example.org")),
			("http://example.org:8080", Some("example.org")),
			("http://user@example.org", Some("example.org")),
			("http://user:pass@example.org:8080", Some("example.org")),
			("http://[::1]", Some("[::1]")),
			("http://[::1]:8080", Some("[::1]")),
			("http://192.168.1.1", Some("192.168.1.1")),
			("mailto:user@example.org", None),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(
				uri.authority().map(|a| a.host().as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn authority_port() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", None),
			("http://example.org:8080", Some("8080")),
			("http://example.org:", Some("")),
			("http://user@example.org:443", Some("443")),
			("http://[::1]:8080", Some("8080")),
			("mailto:user@example.org", None),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(
				uri.authority().and_then(|a| a.port()).map(|p| p.as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn authority_user_info() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", None),
			("http://user@example.org", Some("user")),
			("http://user:pass@example.org", Some("user:pass")),
			("http://user:@example.org", Some("user:")),
			("http://:pass@example.org", Some(":pass")),
			("http://@example.org", Some("")),
			("mailto:user@example.org", None),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(
				uri.authority()
					.and_then(|a| a.user_info())
					.map(|u| u.as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn set_authority() {
		let vectors: &[(&str, Option<&str>, &str)] = &[
			(
				"scheme://authority/path?query#frag",
				Some("authority"),
				"scheme://authority/path?query#frag",
			),
			(
				"scheme://authority/path?query#frag",
				None,
				"scheme:/path?query#frag",
			),
			(
				"scheme://authority/path?query#frag",
				Some("bar"),
				"scheme://bar/path?query#frag",
			),
			(
				"scheme://authority/path?query#frag",
				Some("%61uthority"),
				"scheme://%61uthority/path?query#frag",
			),
			(
				"scheme://%61uthority/path?query#frag",
				Some("bar"),
				"scheme://bar/path?query#frag",
			),
			("scheme:", Some("%61uthority"), "scheme://%61uthority/"),
		];

		for (input_uri, input_authority, expected) in vectors {
			let mut uri = UriBuf::new(input_uri.to_string()).unwrap();
			let authority = input_authority.map(|a| Authority::new(a).unwrap());
			uri.set_authority(authority);
			assert_eq!(
				uri.as_str(),
				*expected,
				"input uri: {input_uri}, authority: {input_authority:?}"
			);
		}
	}

	#[test]
	fn path() {
		let vectors: &[(&str, &str)] = &[
			("http://example.org", ""),
			("http://example.org/", "/"),
			("http://example.org/path", "/path"),
			("http://example.org/path/to/resource", "/path/to/resource"),
			("http://example.org/path?query", "/path"),
			("http://example.org/path#fragment", "/path"),
			("mailto:user@example.org", "user@example.org"),
			("urn:isbn:0451450523", "isbn:0451450523"),
			("file:///path/to/file", "/path/to/file"),
			("http://example.org/a/b/../c", "/a/b/../c"),
			("http://example.org/path/", "/path/"),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(uri.path().as_str(), *expected, "input: {input}");
		}
	}

	#[test]
	fn query() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", None),
			("http://example.org?", Some("")),
			("http://example.org?query", Some("query")),
			("http://example.org?key=value", Some("key=value")),
			(
				"http://example.org?key=value&other=123",
				Some("key=value&other=123"),
			),
			("http://example.org/path?query", Some("query")),
			("http://example.org?query#fragment", Some("query")),
			("http://example.org#fragment", None),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(uri.query().map(|q| q.as_str()), *expected, "input: {input}");
		}
	}

	#[test]
	fn set_query() {
		let vectors: &[(&str, Option<&str>, &str)] = &[
			(
				"scheme://authority/path#frag",
				Some("query"),
				"scheme://authority/path?query#frag",
			),
			(
				"scheme://authority/path?query#frag",
				None,
				"scheme://authority/path#frag",
			),
			(
				"scheme://authority/path#frag",
				Some("%71uery"),
				"scheme://authority/path?%71uery#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				Some("%71uery=%62ar"),
				"scheme://authority/path?%71uery=%62ar#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				Some("query"),
				"scheme://authority/path?query#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				Some("%71uery"),
				"scheme://authority/path?%71uery#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				None,
				"scheme://authority/path#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				Some("%71uery"),
				"scheme://authority/path?%71uery#frag",
			),
			(
				"scheme://authority/path?%71uery#frag",
				None,
				"scheme://authority/path#frag",
			),
		];

		for (input_uri, input_query, expected) in vectors {
			let mut uri = UriBuf::new(input_uri.to_string()).unwrap();
			let query = input_query.map(|q| Query::new(q).unwrap());
			uri.set_query(query);
			assert_eq!(
				uri.as_str(),
				*expected,
				"input uri: {input_uri}, query: {input_query:?}"
			);
		}
	}

	#[test]
	fn fragment() {
		let vectors: &[(&str, Option<&str>)] = &[
			("http://example.org", None),
			("http://example.org#", Some("")),
			("http://example.org#fragment", Some("fragment")),
			("http://example.org#section-1", Some("section-1")),
			("http://example.org/path#fragment", Some("fragment")),
			("http://example.org?query#fragment", Some("fragment")),
			("http://example.org?query", None),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(
				uri.fragment().map(|f| f.as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn set_fragment() {
		let vectors: &[(&str, Option<&str>, &str)] = &[
			(
				"scheme://authority/path",
				Some("frag"),
				"scheme://authority/path#frag",
			),
			(
				"scheme://authority/path#frag",
				None,
				"scheme://authority/path",
			),
			(
				"scheme://authority/path",
				Some("%66rag"),
				"scheme://authority/path#%66rag",
			),
			(
				"scheme://authority/path#%66rag",
				Some("%66rag"),
				"scheme://authority/path#%66rag",
			),
			(
				"scheme://authority/path#%66rag",
				Some("frag"),
				"scheme://authority/path#frag",
			),
			(
				"scheme://authority/path#%66rag",
				Some("%66rag"),
				"scheme://authority/path#%66rag",
			),
			(
				"scheme://authority/path#%66rag",
				None,
				"scheme://authority/path",
			),
			(
				"scheme://authority/path?q#%66rag",
				Some("%66rag"),
				"scheme://authority/path?q#%66rag",
			),
			(
				"scheme://authority/path?q#%66rag",
				None,
				"scheme://authority/path?q",
			),
		];

		for (input_uri, input_fragment, expected) in vectors {
			let mut uri = UriBuf::new(input_uri.to_string()).unwrap();
			let fragment = input_fragment.map(|f| Fragment::new(f).unwrap());
			uri.set_fragment(fragment);
			assert_eq!(
				uri.as_str(),
				*expected,
				"input uri: {input_uri}, fragment: {input_fragment:?}"
			);
		}
	}

	#[test]
	fn parts() {
		let vectors: &[(&str, &str, Option<&str>, &str, Option<&str>, Option<&str>)] = &[
			// (uri, scheme, authority, path, query, fragment)
			(
				"http://example.org",
				"http",
				Some("example.org"),
				"",
				None,
				None,
			),
			(
				"https://user:pass@example.org:8080/path?query#fragment",
				"https",
				Some("user:pass@example.org:8080"),
				"/path",
				Some("query"),
				Some("fragment"),
			),
			(
				"mailto:user@example.org",
				"mailto",
				None,
				"user@example.org",
				None,
				None,
			),
			(
				"urn:isbn:0451450523",
				"urn",
				None,
				"isbn:0451450523",
				None,
				None,
			),
			(
				"file:///etc/passwd",
				"file",
				Some(""),
				"/etc/passwd",
				None,
				None,
			),
			(
				"http://[::1]:8080/",
				"http",
				Some("[::1]:8080"),
				"/",
				None,
				None,
			),
			(
				"http://example.org?",
				"http",
				Some("example.org"),
				"",
				Some(""),
				None,
			),
			(
				"http://example.org#",
				"http",
				Some("example.org"),
				"",
				None,
				Some(""),
			),
			(
				"http://example.org/?#",
				"http",
				Some("example.org"),
				"/",
				Some(""),
				Some(""),
			),
		];

		for (input, scheme, authority, path, query, fragment) in vectors {
			let uri = Uri::new(input).unwrap();
			let parts = uri.parts();

			assert_eq!(
				parts.scheme.as_str(),
				*scheme,
				"scheme mismatch for {input}"
			);
			assert_eq!(
				parts.authority.map(|a| a.as_str()),
				*authority,
				"authority mismatch for {input}"
			);
			assert_eq!(parts.path.as_str(), *path, "path mismatch for {input}");
			assert_eq!(
				parts.query.map(|q| q.as_str()),
				*query,
				"query mismatch for {input}"
			);
			assert_eq!(
				parts.fragment.map(|f| f.as_str()),
				*fragment,
				"fragment mismatch for {input}"
			);
		}
	}

	#[test]
	fn base() {
		let vectors: &[(&str, &str)] = &[
			("http://example.org/a/b/c", "http://example.org/a/b/"),
			("http://example.org/a/b/c/", "http://example.org/a/b/c/"),
			("http://example.org/a", "http://example.org/"),
			("http://example.org/", "http://example.org/"),
			("http://example.org", "http://example.org"),
		];

		for (input, expected) in vectors {
			let uri = Uri::new(input).unwrap();
			assert_eq!(uri.base().as_str(), *expected, "input: {input}");
		}
	}

	#[test]
	fn invalid() {
		let vectors: [&[u8]; _] = [
			b"",                      // empty
			b"://host",               // missing scheme
			b"/path",                 // no scheme (relative)
			b"../..",                 // relative reference
			b"http://host name",      // space in host
			b"http://host\0name",     // null byte
			b"http://[::1",           // unclosed bracket
			b"http://ho st/path",     // space in authority
			b"htt p://host",          // space in scheme
			b"http://host/pa th",     // space in path
			b"http://host?qu ery",    // space in query
			b"http://host#fra gment", // space in fragment
			b"\xff://host",           // invalid byte in scheme
		];

		for input in vectors {
			assert!(Uri::new(input).is_err(), "should reject: {input:?}");
		}
	}
}
