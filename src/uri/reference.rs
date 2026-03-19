use core::{
	cmp::Ordering,
	hash::{Hash, Hasher},
};

use crate::{Port, Uri};

#[cfg(feature = "std")]
use crate::{InvalidUri, PathContext, UriBuf};

use super::{Authority, Fragment, Host, Path, Query, Scheme, UserInfo};

#[cfg(feature = "std")]
use super::{
	AuthorityMut, InvalidAuthority, InvalidFragment, InvalidPath, InvalidQuery, PathBuf, PathMut,
	Segment,
};

/// URI reference.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(super::grammar::UriRef)]
#[newtype(name = "URI reference", ord([u8], &[u8], str, &str))]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String), owned(UriRefBuf, derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct UriRef(str);

/// Individual components of a URI reference.
///
/// Contains references to each component of a URI reference as defined
/// in RFC 3986. Unlike [`UriParts`](super::UriParts), the scheme is optional
/// since URI references may be relative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriRefParts<'a> {
	/// Scheme component, if present (e.g., `https`, `http`, `file`).
	pub scheme: Option<&'a Scheme>,

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

impl Default for &UriRef {
	fn default() -> Self {
		<UriRef>::EMPTY
	}
}

impl UriRef {
	/// Empty URI reference.
	pub const EMPTY: &'static Self = unsafe { Self::new_unchecked("") };

	/// Returns all the parts of this URI reference.
	///
	/// This method parses the URI reference and returns a [`UriRefParts`]
	/// struct containing references to each component: scheme, authority, path,
	/// query, and fragment. Unlike [`Uri::parts`], the scheme is optional.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("//example.org/path?query#fragment").unwrap();
	/// let parts = uri_ref.parts();
	///
	/// assert!(parts.scheme.is_none());
	/// assert_eq!(parts.authority.unwrap(), "example.org");
	/// assert_eq!(parts.path, "/path");
	/// assert_eq!(parts.query.unwrap(), "query");
	/// assert_eq!(parts.fragment.unwrap(), "fragment");
	/// ```
	pub fn parts(&self) -> UriRefParts<'_> {
		let bytes = self.as_bytes();
		let ranges = crate::common::parse::reference_parts(bytes, 0);

		UriRefParts {
			scheme: ranges
				.scheme
				.map(|r| unsafe { Scheme::new_unchecked_from_bytes(&bytes[r]) }),
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

	/// Returns the scheme of the URI reference, if any.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let absolute = UriRef::new("https://example.org/path").unwrap();
	/// assert_eq!(absolute.scheme().unwrap(), "https");
	///
	/// let relative = UriRef::new("/path").unwrap();
	/// assert!(relative.scheme().is_none());
	/// ```
	#[inline]
	pub fn scheme(&self) -> Option<&Scheme> {
		let bytes = self.as_bytes();
		crate::common::parse::find_scheme(bytes, 0)
			.map(|range| unsafe { Scheme::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Adds the given scheme to the reference, returning a URI.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRef, Scheme};
	///
	/// let uri_ref = UriRef::new("//example.org/path").unwrap();
	/// let uri = uri_ref.with_scheme(Scheme::new(b"https").unwrap());
	///
	/// assert_eq!(uri, "https://example.org/path");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_scheme(&self, scheme: &Scheme) -> UriBuf {
		self.to_owned().into_with_scheme(scheme)
	}

	/// Returns a copy of this URI reference with the given authority.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRef, uri::Authority};
	///
	/// let uri_ref = UriRef::new("https://example.org/path").unwrap();
	/// let new = uri_ref.with_authority(Some(Authority::new("other.com").unwrap()));
	/// assert_eq!(new, "https://other.com/path");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_authority(&self, authority: Option<&Authority>) -> UriRefBuf {
		let mut buf = self.to_owned();
		buf.set_authority(authority);
		buf
	}

	/// Returns a copy of this URI reference with the given path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRef, uri::Path};
	///
	/// let uri_ref = UriRef::new("https://example.org/old").unwrap();
	/// let new = uri_ref.with_path(Path::new("/new").unwrap());
	/// assert_eq!(new, "https://example.org/new");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_path(&self, path: &Path) -> UriRefBuf {
		let mut buf = self.to_owned();
		buf.set_path(path);
		buf
	}

	/// Returns a copy of this URI reference with the given query.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRef, uri::Query};
	///
	/// let uri_ref = UriRef::new("https://example.org/path").unwrap();
	/// let new = uri_ref.with_query(Some(Query::new("key=value").unwrap()));
	/// assert_eq!(new, "https://example.org/path?key=value");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_query(&self, query: Option<&Query>) -> UriRefBuf {
		let mut buf = self.to_owned();
		buf.set_query(query);
		buf
	}

	/// Returns a copy of this URI reference with the given fragment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRef, uri::Fragment};
	///
	/// let uri_ref = UriRef::new("https://example.org/path").unwrap();
	/// let new = uri_ref.with_fragment(Some(Fragment::new("section").unwrap()));
	/// assert_eq!(new, "https://example.org/path#section");
	/// ```
	#[cfg(feature = "std")]
	pub fn with_fragment(&self, fragment: Option<&Fragment>) -> UriRefBuf {
		let mut buf = self.to_owned();
		buf.set_fragment(fragment);
		buf
	}

	/// Returns the authority part of the URI reference, if any.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://user@example.org:8080/path").unwrap();
	/// assert_eq!(uri_ref.authority().unwrap(), "user@example.org:8080");
	///
	/// let no_authority = UriRef::new("/path").unwrap();
	/// assert!(no_authority.authority().is_none());
	/// ```
	pub fn authority(&self) -> Option<&Authority> {
		let bytes = self.as_bytes();
		crate::common::parse::find_authority(bytes, 0)
			.ok()
			.map(|range| unsafe { Authority::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Returns the host of the URI reference, if an authority is present.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://example.org:8080/path").unwrap();
	/// assert_eq!(uri_ref.host().unwrap(), "example.org");
	///
	/// let no_authority = UriRef::new("/path").unwrap();
	/// assert!(no_authority.host().is_none());
	/// ```
	pub fn host(&self) -> Option<&Host> {
		self.authority().map(Authority::host)
	}

	/// Returns the user info of the URI reference, if present.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://user:pass@example.org/path").unwrap();
	/// assert_eq!(uri_ref.user_info().unwrap(), "user:pass");
	///
	/// let no_userinfo = UriRef::new("https://example.org/path").unwrap();
	/// assert!(no_userinfo.user_info().is_none());
	/// ```
	pub fn user_info(&self) -> Option<&UserInfo> {
		self.authority().and_then(Authority::user_info)
	}

	/// Returns the port of the URI reference, if present.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://example.org:8080/path").unwrap();
	/// assert_eq!(uri_ref.port().unwrap(), "8080");
	///
	/// let no_port = UriRef::new("https://example.org/path").unwrap();
	/// assert!(no_port.port().is_none());
	/// ```
	pub fn port(&self) -> Option<&Port> {
		self.authority().and_then(Authority::port)
	}

	/// Returns the path of the URI reference.
	///
	/// The path is always present, though it may be empty.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://example.org/foo/bar?query").unwrap();
	/// assert_eq!(uri_ref.path(), "/foo/bar");
	///
	/// let empty_path = UriRef::new("https://example.org").unwrap();
	/// assert_eq!(empty_path.path(), "");
	/// ```
	pub fn path(&self) -> &Path {
		let bytes = self.as_bytes();
		let range = crate::common::parse::find_path(bytes, 0);
		unsafe { Path::new_unchecked_from_bytes(&bytes[range]) }
	}

	/// Returns the query component of the URI reference, if any.
	pub fn query(&self) -> Option<&Query> {
		let bytes = self.as_bytes();
		crate::common::parse::find_query(bytes, 0)
			.ok()
			.map(|range| unsafe { Query::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Returns the fragment component of the URI reference, if any.
	pub fn fragment(&self) -> Option<&Fragment> {
		let bytes = self.as_bytes();
		crate::common::parse::find_fragment(bytes, 0)
			.ok()
			.map(|range| unsafe { Fragment::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Returns this URI reference without the fragment component.
	///
	/// If the URI reference has a fragment (e.g. `#section`), it is removed
	/// along with the `#` delimiter. If there is no fragment, the original
	/// URI reference is returned unchanged.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://example.org/path?query#frag").unwrap();
	/// assert_eq!(uri_ref.without_fragment(), "https://example.org/path?query");
	///
	/// let no_frag = UriRef::new("https://example.org/path").unwrap();
	/// assert_eq!(no_frag.without_fragment(), "https://example.org/path");
	/// ```
	pub fn without_fragment(&self) -> &Self {
		let bytes = self.as_bytes();
		match crate::common::parse::find_fragment(bytes, 0) {
			Ok(range) => unsafe { Self::new_unchecked_from_bytes(&bytes[..range.start - 1]) },
			Err(_) => self,
		}
	}

	/// Returns this URI reference without the query and fragment components.
	///
	/// If the URI reference has a query (e.g. `?key=value`) or fragment
	/// (e.g. `#section`), they are removed along with their delimiters.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri_ref = UriRef::new("https://example.org/path?query#frag").unwrap();
	/// assert_eq!(
	///     uri_ref.without_query_and_fragment(),
	///     "https://example.org/path"
	/// );
	///
	/// let no_query = UriRef::new("https://example.org/path#frag").unwrap();
	/// assert_eq!(
	///     no_query.without_query_and_fragment(),
	///     "https://example.org/path"
	/// );
	/// ```
	pub fn without_query_and_fragment(&self) -> &Self {
		let bytes = self.as_bytes();
		let end = match crate::common::parse::find_query(bytes, 0) {
			Ok(range) => range.start - 1,
			Err(i) => i,
		};
		unsafe { Self::new_unchecked_from_bytes(&bytes[..end]) }
	}

	/// Returns this URI reference relative to the given base.
	///
	/// Computes a relative URI reference that, when resolved against `other`,
	/// would produce `self`. This is the inverse operation of resolution.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let base = UriRef::new("https://example.org/foo/bar").unwrap();
	/// let target = UriRef::new("https://example.org/foo/baz").unwrap();
	///
	/// assert_eq!(target.relative_to(base), "baz");
	///
	/// let other = UriRef::new("https://example.org/other").unwrap();
	/// assert_eq!(other.relative_to(base), "../other");
	/// ```
	#[cfg(feature = "std")]
	#[inline]
	pub fn relative_to(&self, other: &Self) -> UriRefBuf {
		let mut result = <UriRefBuf>::default();

		match (self.scheme(), other.scheme()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => {
				return unsafe { <UriRefBuf>::new_unchecked(self.as_bytes().to_vec()) };
			}
		}

		match (self.authority(), other.authority()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => {
				return unsafe { <UriRefBuf>::new_unchecked(self.as_bytes().to_vec()) };
			}
		}

		let self_path = self.path();
		let base_path = other.path();

		let mut self_segments = self_path.normalized_segments().peekable();
		let mut base_segments = base_path.parent_or_empty().normalized_segments().peekable();

		if self_path.is_absolute() == base_path.is_absolute() {
			loop {
				match (self_segments.peek(), base_segments.peek()) {
					(Some(a), Some(b)) if a == b => {
						base_segments.next();
						self_segments.next();
					}
					_ => break,
				}
			}
		}

		for _ in base_segments {
			result.path_mut().lazy_push(Segment::PARENT);
		}

		if self_path.is_absolute() && base_path == Path::EMPTY_RELATIVE {
			result.set_path(Path::EMPTY_ABSOLUTE);
		}

		for segment in self_segments {
			result.path_mut().lazy_push(segment);
		}

		if (self.query().is_some() || self.fragment().is_some())
			&& Some(result.path().as_bytes()) == other.path().last().map(|s| s.as_bytes())
		{
			result.path_mut().clear();
		}

		result.set_query(self.query());
		result.set_fragment(self.fragment());

		result
	}

	/// Returns the suffix of this URI relative to the given prefix.
	///
	/// Returns `Some((suffix, query, fragment))` if this URI is of the form
	/// `prefix/suffix?query#fragment` where `prefix` is given as parameter.
	/// Returns `None` otherwise.
	/// If the scheme or authority differs from the prefix, returns `None`.
	///
	/// See [`Path::suffix`] for more details.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let uri = UriRef::new("https://example.org/foo/bar/baz?query").unwrap();
	/// let prefix = UriRef::new("https://example.org/foo").unwrap();
	///
	/// let (suffix, query, fragment) = uri.suffix(prefix).unwrap();
	/// assert_eq!(suffix, "bar/baz");
	/// assert_eq!(query.unwrap(), "query");
	/// assert!(fragment.is_none());
	/// ```
	#[cfg(feature = "std")]
	#[inline]
	pub fn suffix(
		&self,
		prefix: impl AsRef<Self>,
	) -> Option<(PathBuf, Option<&Query>, Option<&Fragment>)> {
		let prefix = prefix.as_ref();
		if self.scheme() == prefix.scheme() && self.authority() == prefix.authority() {
			self.path()
				.suffix(prefix.path())
				.map(|suffix_path| (suffix_path, self.query(), self.fragment()))
		} else {
			None
		}
	}

	/// The IRI reference without the file name, query and fragment.
	///
	/// # Example
	/// ```
	/// # use iref::IriRef;
	/// let a = IriRef::new("https://crates.io/crates/iref?query#fragment").unwrap();
	/// let b = IriRef::new("https://crates.io/crates/iref/?query#fragment").unwrap();
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

	/// Resolves the URI reference against the given base URI.
	///
	/// See the [`UriRefBuf::resolve`] method for more information about the
	/// resolution process.
	#[cfg(feature = "std")]
	#[inline]
	pub fn resolved(&self, base_iri: impl AsRef<Uri>) -> UriBuf {
		let iri_ref = self.to_owned();
		iri_ref.into_resolved(base_iri)
	}

	/// Resolves the URI reference against the given base URI.
	///
	/// Same as [`Self::resolved`] but accepts a `&str` instead of an
	/// URI. Returns an error if the input is not a valid URI.
	#[cfg(feature = "std")]
	pub fn try_resolved<'r>(
		&self,
		base_iri: &'r str,
	) -> Result<UriBuf, <&'r Uri as TryFrom<&'r str>>::Error> {
		Uri::new(base_iri).map(|u| self.resolved(u))
	}

	/// Returns this URI reference as a URI, if it has a scheme.
	///
	/// A URI reference with a scheme is a valid URI.
	/// Returns `None` if this is a relative reference (no scheme).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRef;
	///
	/// let absolute = UriRef::new("https://example.org/path").unwrap();
	/// assert!(absolute.as_uri().is_some());
	///
	/// let relative = UriRef::new("/path").unwrap();
	/// assert!(relative.as_uri().is_none());
	/// ```
	#[inline]
	pub fn as_uri(&self) -> Option<&Uri> {
		if self.scheme().is_some() {
			Some(unsafe { Uri::new_unchecked(&self.0) })
		} else {
			None
		}
	}
}

/// Parses an [`UriRef`] at compile time.
#[macro_export]
macro_rules! uri_ref {
	($value:literal) => {
		match $crate::uri::UriRef::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI reference"),
		}
	};
}

impl PartialEq for UriRef {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a UriRef> for UriRef {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl PartialEq<Uri> for UriRef {
	fn eq(&self, other: &Uri) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl<'a> PartialEq<&'a Uri> for UriRef {
	fn eq(&self, other: &&'a Uri) -> bool {
		*self == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialEq<UriBuf> for UriRef {
	fn eq(&self, other: &UriBuf) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl Eq for UriRef {}

impl PartialOrd for UriRef {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a UriRef> for UriRef {
	fn partial_cmp(&self, other: &&'a Self) -> Option<Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<Uri> for UriRef {
	fn partial_cmp(&self, other: &Uri) -> Option<Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl<'a> PartialOrd<&'a Uri> for UriRef {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

#[cfg(feature = "std")]
impl PartialOrd<UriBuf> for UriRef {
	fn partial_cmp(&self, other: &UriBuf) -> Option<Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl Ord for UriRef {
	fn cmp(&self, other: &Self) -> Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for UriRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl<'a> From<&'a Uri> for &'a UriRef {
	fn from(value: &'a Uri) -> Self {
		value.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl UriRefBuf {
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
	/// use iref::UriRefBuf;
	///
	/// let mut uri_ref = UriRefBuf::new("//example.org:8080/path".to_string()).unwrap();
	///
	/// if let Some(mut authority) = uri_ref.authority_mut() {
	///     authority.set_host("other.com".try_into().unwrap());
	/// }
	///
	/// assert_eq!(uri_ref, "//other.com:8080/path");
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
	/// use iref::{IriRefBuf, iri::Authority};
	///
	/// let mut a = IriRefBuf::new("scheme:/path".to_string()).unwrap();
	/// a.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(a, "scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = IriRefBuf::new("scheme:path".to_string()).unwrap();
	/// b.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(b, "scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = IriRefBuf::new("scheme://example.org//path".to_string()).unwrap();
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
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRefBuf, uri::Authority};
	///
	/// let mut uri_ref = UriRefBuf::new("/path".to_string()).unwrap();
	///
	/// // Set authority
	/// uri_ref.try_set_authority(Some("example.org")).unwrap();
	/// assert_eq!(uri_ref, "//example.org/path");
	///
	/// // Remove authority
	/// uri_ref.try_set_authority(None).unwrap();
	/// assert_eq!(uri_ref, "/path");
	/// ```
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
	/// use iref::{UriRefBuf, uri::Segment};
	///
	/// let mut uri_ref = UriRefBuf::new("/foo/../bar?query".to_string()).unwrap();
	/// uri_ref.path_mut()
	///     .normalize()
	///     .try_push("baz")
	///     .unwrap();
	/// assert_eq!(uri_ref, "/bar/baz?query");
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
	/// use iref::{IriRefBuf, iri::Path};
	///
	/// let mut a = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// a.set_path(Path::new("/foo/bar").unwrap());
	/// assert_eq!(a, "http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// b.set_path(Path::new("relative/path").unwrap());
	/// assert_eq!(b, "http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = IriRefBuf::new("http:old/path".to_string()).unwrap();
	/// c.set_path(Path::new("//foo/bar").unwrap());
	/// assert_eq!(c, "http:/.//foo/bar");
	///
	/// // If there is no authority nor scheme, and the path beginning looks
	/// // like a scheme, it is prefixed with `./` to avoid being confused with
	/// // a scheme.
	/// let mut d = IriRefBuf::new("old/path".to_string()).unwrap();
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
	/// use iref::{UriRefBuf, uri::Query};
	///
	/// let mut uri_ref = UriRefBuf::new("/path".to_string()).unwrap();
	///
	/// uri_ref.set_query(Some(Query::new("key=value").unwrap()));
	/// assert_eq!(uri_ref, "/path?key=value");
	///
	/// uri_ref.set_query(None);
	/// assert_eq!(uri_ref, "/path");
	/// ```
	#[inline]
	pub fn set_query(&mut self, query: Option<&Query>) {
		match query {
			Some(new_query) => match crate::common::parse::find_query(self.as_bytes(), 0) {
				Ok(range) => unsafe { self.replace(range, new_query.as_bytes()) },
				Err(start) => unsafe {
					self.allocate(start..start, new_query.len() + 1);
					let bytes = self.as_mut_vec();
					let delim_end = start + 1;
					bytes[start] = b'?';
					bytes[delim_end..(delim_end + new_query.len())]
						.copy_from_slice(new_query.as_bytes())
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
	/// use iref::{UriRefBuf, uri::Fragment};
	///
	/// let mut uri_ref = UriRefBuf::new("/path".to_string()).unwrap();
	///
	/// uri_ref.set_fragment(Some(Fragment::new("section").unwrap()));
	/// assert_eq!(uri_ref, "/path#section");
	///
	/// uri_ref.set_fragment(None);
	/// assert_eq!(uri_ref, "/path");
	/// ```
	#[inline]
	pub fn set_fragment(&mut self, fragment: Option<&Fragment>) {
		match fragment {
			Some(new_fragment) => match crate::common::parse::find_fragment(self.as_bytes(), 0) {
				Ok(range) => unsafe { self.replace(range, new_fragment.as_bytes()) },
				Err(start) => unsafe {
					self.allocate(start..start, new_fragment.len() + 1);
					let bytes = self.as_mut_vec();
					let delim_end = start + 1;
					bytes[start] = b'#';
					bytes[delim_end..(delim_end + new_fragment.len())]
						.copy_from_slice(new_fragment.as_bytes())
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

	/// Sets the scheme part.
	///
	/// If there is no authority and the start of the path looks like a scheme
	/// (e.g. `foo:`) then the path is prefixed with `./` to avoid being
	/// confused with a scheme.
	///
	/// # Example
	///
	/// ```
	/// use iref::{IriRefBuf, iri::Scheme};
	///
	/// let mut a = IriRefBuf::new("foo/bar".to_string()).unwrap();
	/// a.set_scheme(Some(Scheme::new(b"http").unwrap()));
	/// assert_eq!(a, "http:foo/bar");
	///
	/// let mut b = IriRefBuf::new("scheme://example.org/foo/bar".to_string()).unwrap();
	/// b.set_scheme(None);
	/// assert_eq!(b, "//example.org/foo/bar");
	///
	/// let mut c = IriRefBuf::new("scheme:foo:bar".to_string()).unwrap();
	/// c.set_scheme(None);
	/// assert_eq!(c, "./foo:bar");
	/// ```
	#[inline]
	pub fn set_scheme(&mut self, scheme: Option<&Scheme>) {
		match scheme {
			Some(new_scheme) => match crate::common::parse::find_scheme(self.as_bytes(), 0) {
				Some(scheme_range) => unsafe {
					self.replace(scheme_range, new_scheme.as_bytes());
				},
				None => unsafe {
					self.allocate(0..0, new_scheme.len() + 1);
					let bytes = self.as_mut_vec();
					bytes[0..new_scheme.len()].copy_from_slice(new_scheme.as_bytes());
					bytes[new_scheme.len()] = b':'
				},
			},
			None => {
				if let Some(scheme_range) = crate::common::parse::find_scheme(self.as_bytes(), 0) {
					let value: &[u8] =
						if self.authority().is_none() && self.path().looks_like_scheme() {
							// AMBIGUITY: The URI `http:foo:bar` would become
							//            `foo:bar`, but `foo` is not the scheme.
							// SOLUTION:  We change `foo:bar` to `./foo:bar`.
							b"./"
						} else {
							b""
						};

					unsafe { self.replace(scheme_range.start..(scheme_range.end + 1), value) }
				}
			}
		}
	}

	/// Tries to set the scheme part.
	///
	/// Same [`Self::set_scheme`] but accepts an `&str` instead of a
	/// [`&Scheme`](Scheme). Returns an error if the input string is not
	/// a valid scheme.
	pub fn try_set_scheme<'s>(
		&mut self,
		scheme: Option<&'s str>,
	) -> Result<(), super::InvalidScheme<&'s str>> {
		self.set_scheme(scheme.map(TryInto::try_into).transpose()?);
		Ok(())
	}

	/// Adds the given scheme to the reference, turning it into an URI.
	pub fn into_with_scheme(mut self, scheme: &Scheme) -> UriBuf {
		self.set_scheme(Some(scheme));
		unsafe { UriBuf::new_unchecked(self.0) }
	}

	/// Tries to add the given scheme to the reference, turning it into
	/// an URI.
	///
	/// Same [`Self::into_with_scheme`] but accepts an `&str` instead of
	/// a [`&Scheme`](Scheme). Returns an error if the input string is
	/// not a valid scheme.
	pub fn try_into_with_scheme(
		mut self,
		scheme: &str,
	) -> Result<UriBuf, (Self, super::InvalidScheme<&str>)> {
		match self.try_set_scheme(Some(scheme)) {
			Ok(_) => Ok(unsafe { UriBuf::new_unchecked(self.0) }),
			Err(e) => Err((self, e)),
		}
	}

	/// Resolves this URI reference against the given base URI in place.
	///
	/// This transforms a relative URI reference into an absolute URI by
	/// resolving it against the provided base URI, following RFC 3986.
	/// This is similar to [`UriBuf::join`], but with the subject and object
	/// swapped.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRefBuf, Uri};
	///
	/// let base = Uri::new("https://example.org/foo/bar").unwrap();
	/// let mut uri_ref = UriRefBuf::new("../baz?query".to_string()).unwrap();
	/// uri_ref.resolve(base);
	///
	/// assert_eq!(uri_ref, "https://example.org/baz?query");
	/// ```
	///
	/// ## Abnormal use of dot segments
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn resolve(&mut self, base_iri: impl AsRef<Uri>) {
		let base_iri = base_iri.as_ref();
		let parts = crate::common::parse::reference_parts(self.as_bytes(), 0);

		if parts.scheme.is_some() {
			self.path_mut().normalize();
		} else {
			self.set_scheme(Some(base_iri.scheme()));
			if parts.authority.is_some() {
				self.path_mut().normalize();
			} else if self.path().is_relative() && self.path().is_empty() {
				self.set_authority(base_iri.authority());
				self.set_path(base_iri.path());
				if self.query().is_none() {
					self.set_query(base_iri.query());
				}
			} else if self.path().is_absolute() {
				self.set_authority(base_iri.authority());
				self.path_mut().normalize();
			} else {
				self.set_authority(base_iri.authority());

				let base_path = base_iri.path();
				let mut path = if base_path.is_empty() && base_iri.authority().is_some() {
					Path::EMPTY_ABSOLUTE.to_owned()
				} else {
					base_path.parent_or_empty().to_owned()
				};

				let mut path_mut = PathMut::from_path_with_context(
					&mut path,
					PathContext {
						// Set the context manually to prevent disambiguation logic.
						has_scheme: false,
						has_authority: false,
					},
				);

				for s in self.path().segments() {
					path_mut.lazy_push(s);
				}

				path_mut.normalize();

				self.set_path(&path);
			}
		}
	}

	/// Tries to resolve this URI reference against the given base URI string.
	///
	/// Same as [`Self::resolve`] but accepts a `&str` instead of a [`&Uri`](Uri).
	/// Returns an error if the base string is not a valid URI.
	pub fn try_resolve<'r>(
		&mut self,
		base_iri: &'r str,
	) -> Result<(), <&'r Uri as TryFrom<&'r str>>::Error> {
		self.resolve(Uri::new(base_iri)?);
		Ok(())
	}

	/// Resolves this URI reference against the given base URI, consuming self.
	///
	/// Returns the resolved URI as an owned [`UriBuf`].
	///
	/// # Example
	///
	/// ```rust
	/// use iref::{UriRefBuf, Uri};
	///
	/// let base = Uri::new("https://example.org/foo/bar").unwrap();
	/// let uri_ref = UriRefBuf::new("../baz".to_string()).unwrap();
	/// let resolved = uri_ref.into_resolved(base);
	///
	/// assert_eq!(resolved, "https://example.org/baz");
	/// ```
	pub fn into_resolved(mut self, base_iri: impl AsRef<Uri>) -> UriBuf {
		self.resolve(base_iri);
		unsafe { <UriBuf>::new_unchecked(self.into_bytes()) }
	}

	/// Tries to resolve this URI reference, consuming self.
	///
	/// Same as [`Self::into_resolved`] but accepts a `&str` instead of a [`&Uri`](Uri).
	/// Returns an error if the base string is not a valid URI.
	pub fn try_into_resolved(
		mut self,
		base_iri: &str,
	) -> Result<UriBuf, (Self, <&Uri as TryFrom<&str>>::Error)> {
		match self.try_resolve(base_iri) {
			Ok(_) => Ok(unsafe { <UriBuf>::new_unchecked(self.into_bytes()) }),
			Err(e) => Err((self, e)),
		}
	}

	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the URI reference.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid URI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		unsafe { self.0.as_mut_vec() }
	}

	/// Tries to convert this URI reference into a URI.
	///
	/// Succeeds if this URI reference has a scheme (making it a valid URI).
	/// Returns an error containing `self` if it's a relative reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::UriRefBuf;
	///
	/// let absolute = UriRefBuf::new("https://example.org/path".to_string()).unwrap();
	/// assert!(absolute.try_into_uri().is_ok());
	///
	/// let relative = UriRefBuf::new("/path".to_string()).unwrap();
	/// assert!(relative.try_into_uri().is_err());
	/// ```
	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(UriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidUri(self))
		}
	}
}

#[cfg(feature = "std")]
impl TryFrom<UriRefBuf> for UriBuf {
	type Error = InvalidUri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

#[cfg(feature = "std")]
impl PartialEq<Uri> for UriRefBuf {
	fn eq(&self, other: &Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl<'a> PartialEq<&'a Uri> for UriRefBuf {
	fn eq(&self, other: &&'a Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialEq<UriBuf> for UriRefBuf {
	fn eq(&self, other: &UriBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

#[cfg(feature = "std")]
impl PartialOrd<Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &Uri) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

#[cfg(feature = "std")]
impl<'a> PartialOrd<&'a Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

#[cfg(feature = "std")]
impl PartialOrd<UriBuf> for UriRefBuf {
	fn partial_cmp(&self, other: &UriBuf) -> Option<Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PARTS: [(
		&[u8],
		(
			Option<&[u8]>,
			Option<&[u8]>,
			&[u8],
			Option<&[u8]>,
			Option<&[u8]>,
		),
	); 38] = [
		// 0 components.
		(b"", (None, None, b"", None, None)),
		(b"a/:", (None, None, b"a/:", None, None)),
		// 1 component.
		(b"scheme:", (Some(b"scheme"), None, b"", None, None)),
		(b"//authority", (None, Some(b"authority"), b"", None, None)),
		(b"path", (None, None, b"path", None, None)),
		(b"/path", (None, None, b"/path", None, None)),
		(b"/", (None, None, b"/", None, None)),
		(b"foo//bar", (None, None, b"foo//bar", None, None)),
		(b"?query", (None, None, b"", Some(b"query"), None)),
		(b"#fragment", (None, None, b"", None, Some(b"fragment"))),
		(
			b"scheme:?query",
			(Some(b"scheme"), None, b"", Some(b"query"), None),
		),
		(b"//[::]", (None, Some(b"[::]"), b"", None, None)),
		// 2 components.
		(
			b"scheme://authority",
			(Some(b"scheme"), Some(b"authority"), b"", None, None),
		),
		(b"scheme:path", (Some(b"scheme"), None, b"path", None, None)),
		(
			b"scheme:/path",
			(Some(b"scheme"), None, b"/path", None, None),
		),
		(
			b"scheme:?query",
			(Some(b"scheme"), None, b"", Some(b"query"), None),
		),
		(
			b"scheme:#fragment",
			(Some(b"scheme"), None, b"", None, Some(b"fragment")),
		),
		(
			b"//authority/path",
			(None, Some(b"authority"), b"/path", None, None),
		),
		(
			b"//authority?query",
			(None, Some(b"authority"), b"", Some(b"query"), None),
		),
		(
			b"//authority#fragment",
			(None, Some(b"authority"), b"", None, Some(b"fragment")),
		),
		(b"path?query", (None, None, b"path", Some(b"query"), None)),
		(b"/path?query", (None, None, b"/path", Some(b"query"), None)),
		(
			b"path#fragment",
			(None, None, b"path", None, Some(b"fragment")),
		),
		(
			b"?query#fragment",
			(None, None, b"", Some(b"query"), Some(b"fragment")),
		),
		// 3 components
		(
			b"scheme://authority/path",
			(Some(b"scheme"), Some(b"authority"), b"/path", None, None),
		),
		(
			b"scheme://authority?query",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				Some(b"query"),
				None,
			),
		),
		(
			b"scheme://authority#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				None,
				Some(b"fragment"),
			),
		),
		(
			b"scheme:path?query",
			(Some(b"scheme"), None, b"path", Some(b"query"), None),
		),
		(
			b"scheme:path#fragment",
			(Some(b"scheme"), None, b"path", None, Some(b"fragment")),
		),
		(
			b"//authority/path?query",
			(None, Some(b"authority"), b"/path", Some(b"query"), None),
		),
		(
			b"//authority/path#fragment",
			(None, Some(b"authority"), b"/path", None, Some(b"fragment")),
		),
		(
			b"//authority?query#fragment",
			(
				None,
				Some(b"authority"),
				b"",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		(
			b"path?query#fragment",
			(None, None, b"path", Some(b"query"), Some(b"fragment")),
		),
		// 4 components
		(
			b"scheme://authority/path?query",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				Some(b"query"),
				None,
			),
		),
		(
			b"scheme://authority/path#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				None,
				Some(b"fragment"),
			),
		),
		(
			b"scheme://authority?query#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		(
			b"scheme:path?query#fragment",
			(
				Some(b"scheme"),
				None,
				b"path",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		// 5 components
		(
			b"scheme://authority/path?query#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
	];

	#[test]
	fn parts() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.scheme.map(Scheme::as_bytes), expected.0);
			assert_eq!(parts.authority.map(Authority::as_bytes), expected.1);
			assert_eq!(parts.path.as_bytes(), expected.2);
			assert_eq!(parts.query.map(Query::as_bytes), expected.3);
			assert_eq!(parts.fragment.map(Fragment::as_bytes), expected.4)
		}
	}

	#[test]
	fn scheme() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			assert_eq!(input.scheme().map(Scheme::as_bytes), expected.0)
		}
	}

	#[test]
	fn authority() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.authority().map(Authority::as_bytes), expected.1)
		}
	}

	#[test]
	fn authority_port() {
		let vectors: &[(&str, Option<&str>)] = &[("//[::]", None)];

		for (input, expected) in vectors {
			let uri = UriRef::new(input).unwrap();
			assert_eq!(
				uri.authority().and_then(|a| a.port()).map(|p| p.as_str()),
				*expected,
				"input: {input}"
			);
		}
	}

	#[test]
	fn set_authority() {
		let vectors: [(&[u8], Option<&[u8]>, &[u8]); 3] = [
			(
				b"scheme:/path",
				Some(b"authority"),
				b"scheme://authority/path",
			),
			(
				b"scheme:path",
				Some(b"authority"),
				b"scheme://authority/path",
			),
			(b"scheme://authority//path", None, b"scheme:/.//path"),
		];

		for (input, authority, expected) in vectors {
			let mut buffer = UriRefBuf::new(input.to_vec()).unwrap();
			let authority = authority.map(Authority::new).transpose().unwrap();
			buffer.set_authority(authority);
			// eprintln!("{input:?}, {authority:?} => {buffer}, {expected:?}");
			assert_eq!(buffer.as_bytes(), expected)
		}
	}

	#[test]
	fn path() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.path().as_bytes(), expected.2)
		}
	}

	#[test]
	fn query() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.query().map(Query::as_bytes), expected.3)
		}
	}

	#[test]
	fn fragment() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.fragment().map(Fragment::as_bytes), expected.4)
		}
	}

	#[test]
	fn disambiguate_scheme() {
		let mut uri_ref = UriRefBuf::new("scheme:a:b/c".to_string()).unwrap();
		uri_ref.set_scheme(None);
		assert_eq!(uri_ref.as_str(), "./a:b/c")
	}

	#[test]
	fn disambiguate_authority() {
		let mut uri_ref = UriRefBuf::new("//host//path".to_string()).unwrap();
		uri_ref.set_authority(None);
		assert_eq!(uri_ref.as_str(), "/.//path")
	}

	fn test_resolution<'a>(base: &str, vectors: impl IntoIterator<Item = (&'a str, &'a str)>) {
		let base_uri = Uri::new(base).unwrap();
		for (reference, expected) in vectors {
			let uri_ref = UriRef::new(reference).unwrap();
			let expected_uri = Uri::new(expected).unwrap();

			assert_eq!(
				uri_ref.resolved(base_uri),
				expected_uri,
				"({uri_ref}).resolved({base_uri})",
			);
			assert_eq!(
				base_uri.joined(uri_ref),
				expected_uri,
				"({base_uri}).joined({uri_ref})",
			);
		}
	}

	/// RFC 3986 Section 5.4 — Normal examples.
	/// https://www.w3.org/2004/04/uri-rel-test.html
	#[test]
	fn resolution_normal() {
		test_resolution(
			"http://a/b/c/d;p?q",
			[
				("g:h", "g:h"),
				("g", "http://a/b/c/g"),
				("./g", "http://a/b/c/g"),
				("g/", "http://a/b/c/g/"),
				("/g", "http://a/g"),
				("//g", "http://g"),
				("?y", "http://a/b/c/d;p?y"),
				("g?y", "http://a/b/c/g?y"),
				("#s", "http://a/b/c/d;p?q#s"),
				("g#s", "http://a/b/c/g#s"),
				("g?y#s", "http://a/b/c/g?y#s"),
				(";x", "http://a/b/c/;x"),
				("g;x", "http://a/b/c/g;x"),
				("g;x?y#s", "http://a/b/c/g;x?y#s"),
				("", "http://a/b/c/d;p?q"),
				(".", "http://a/b/c/"),
				("./", "http://a/b/c/"),
				("..", "http://a/b/"),
				("../", "http://a/b/"),
				("../g", "http://a/b/g"),
				("../..", "http://a/"),
				("../../", "http://a/"),
				("../../g", "http://a/g"),
			],
		);
	}

	/// RFC 3986 Section 5.4 — Abnormal examples.
	/// NOTE we implement [Errata 4547](https://www.rfc-editor.org/errata/eid4547).
	#[test]
	fn resolution_abnormal() {
		test_resolution(
			"http://a/b/c/d;p?q",
			[
				("../../../g", "http://a/g"),
				("../../../../g", "http://a/g"),
				("/./g", "http://a/g"),
				("/../g", "http://a/g"),
				("g.", "http://a/b/c/g."),
				(".g", "http://a/b/c/.g"),
				("g..", "http://a/b/c/g.."),
				("..g", "http://a/b/c/..g"),
				("./../g", "http://a/b/g"),
				("./g/.", "http://a/b/c/g/"),
				("g/./h", "http://a/b/c/g/h"),
				("g/../h", "http://a/b/c/h"),
				("g;x=1/./y", "http://a/b/c/g;x=1/y"),
				("g;x=1/../y", "http://a/b/c/y"),
				("g?y/./x", "http://a/b/c/g?y/./x"),
				("g?y/../x", "http://a/b/c/g?y/../x"),
				("g#s/./x", "http://a/b/c/g#s/./x"),
				("g#s/../x", "http://a/b/c/g#s/../x"),
				("http:g", "http:g"),
			],
		);
	}

	#[test]
	fn resolution_ambiguous_double_slash() {
		test_resolution("http:/a/b", [("../..//", "http:/.//")]);
	}

	#[test]
	fn resolution_longer_segments() {
		test_resolution(
			"http://a/bb/ccc/d;p?q",
			[
				("#s", "http://a/bb/ccc/d;p?q#s"),
				("", "http://a/bb/ccc/d;p?q"),
			],
		);
	}

	#[test]
	fn resolution_dot_segments_in_base() {
		test_resolution(
			"http://a/bb/ccc/./d;p?q",
			[
				("..", "http://a/bb/"),
				("../", "http://a/bb/"),
				("../g", "http://a/bb/g"),
				("../..", "http://a/"),
				("../../", "http://a/"),
				("../../g", "http://a/g"),
			],
		);
	}

	#[test]
	fn resolution_double_slashes_in_base() {
		test_resolution(
			"http://ab//de//ghi",
			[
				("xyz", "http://ab//de//xyz"),
				("./xyz", "http://ab//de//xyz"),
				("../xyz", "http://ab//de/xyz"),
			],
		);
	}

	#[test]
	fn resolution_parent_segments_in_base() {
		test_resolution("http://a/bb/ccc/../d;p?q", [("../../", "http://a/")]);
	}

	/// https://github.com/timothee-haudebourg/iref/issues/14
	#[test]
	fn resolution_scheme_no_authority() {
		test_resolution("scheme:a:b/", [("Foo", "scheme:a:b/Foo")]);
	}

	/// RFC 3986 Section 5.2.3: base URI with authority and empty path.
	#[test]
	fn resolution_empty_base_path() {
		test_resolution(
			"http://a",
			[
				(".", "http://a/"),
				("./", "http://a/"),
				("..", "http://a/"),
				("../", "http://a/"),
				("../g", "http://a/g"),
				("g", "http://a/g"),
				("g/", "http://a/g/"),
				("g/h", "http://a/g/h"),
				("?q", "http://a?q"),
				("#f", "http://a#f"),
			],
		);
	}

	fn test_relative_to<'a>(base: &str, vectors: impl IntoIterator<Item = (&'a str, &'a str)>) {
		let base = UriRef::new(base).unwrap();
		for (input, expected) in vectors {
			let input = UriRef::new(input).unwrap();
			assert_eq!(
				input.relative_to(base),
				expected,
				"({input}).relative_to({base}) != {expected}",
			);
		}
	}

	#[test]
	fn relative_to() {
		test_relative_to(
			"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld",
			[
				(
					"https://w3c.github.io/json-ld-api/tests/compact/link",
					"link",
				),
				(
					"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld#fragment-works",
					"#fragment-works",
				),
				(
					"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld?query=works",
					"?query=works",
				),
				("https://w3c.github.io/json-ld-api/tests/", "../"),
				("https://w3c.github.io/json-ld-api/", "../../"),
				("https://w3c.github.io/json-ld-api/parent", "../../parent"),
				(
					"https://w3c.github.io/json-ld-api/parent#fragment",
					"../../parent#fragment",
				),
				(
					"https://w3c.github.io/parent-parent-eq-root",
					"../../../parent-parent-eq-root",
				),
				(
					"http://example.org/scheme-relative",
					"http://example.org/scheme-relative",
				),
				(
					"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld",
					"0066-in.jsonld",
				),
			],
		);
	}

	#[test]
	fn relative_to_empty_base_path() {
		test_relative_to(
			"http://a",
			[
				("http://a/", "/"),
				("http://a/g", "/g"),
				("http://a/g/h", "/g/h"),
				("http://a?q", "?q"),
				("http://a#f", "#f"),
				("http://a?q#f", "?q#f"),
			],
		);
	}

	#[test]
	fn relative_to_root_base_path() {
		test_relative_to(
			"http://a/",
			[
				("http://a/", ""),
				("http://a/g", "g"),
				("http://a/g/h", "g/h"),
				("http://a/?q", "?q"),
				("http://a/#f", "#f"),
			],
		);
	}

	#[test]
	fn relative_to_different_authority() {
		test_relative_to(
			"http://a/path",
			[
				("http://b/path", "http://b/path"),
				("http://b/other", "http://b/other"),
			],
		);
	}

	#[test]
	fn relative_to_same_uri() {
		test_relative_to(
			"http://a/b/c",
			[
				("http://a/b/c", "c"),
				("http://a/b/c?q", "?q"),
				("http://a/b/c#f", "#f"),
			],
		);
	}

	#[test]
	fn invalid() {
		let vectors: [&[u8]; _] = [
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
			assert!(UriRef::new(input).is_err(), "should reject: {input:?}");
		}
	}

	#[test]
	fn valid_relative() {
		let vectors = [
			"",            // empty is valid
			"/path",       // absolute path
			"../..",       // relative reference
			"?query",      // query only
			"#fragment",   // fragment only
			"//authority", // authority without scheme
		];

		for input in vectors {
			assert!(UriRef::new(input).is_ok(), "should accept: {input:?}");
		}
	}
}
