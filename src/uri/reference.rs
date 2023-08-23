use std::hash::{self, Hash};

use static_regular_grammar::RegularGrammar;

use crate::{
	common::{parse, RiRefBufImpl, RiRefImpl},
	InvalidIri, InvalidUri, Iri, IriBuf, IriRef, IriRefBuf, Uri, UriBuf,
};

use super::{bytestr_eq, Authority, AuthorityMut, Fragment, Path, PathBuf, PathMut, Query, Scheme};

/// URI reference.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "URI-reference",
	name = "URI reference",
	cache = "automata/uri/reference.aut.cbor",
	ascii
)]
#[grammar(sized(
	UriRefBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct UriRef([u8]);

impl RiRefImpl for UriRef {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = UriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriRefParts<'a> {
	pub scheme: Option<&'a Scheme>,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl UriRef {
	pub fn parts(&self) -> UriRefParts {
		let bytes = self.as_bytes();
		let ranges = parse::reference_parts(bytes, 0);

		UriRefParts {
			scheme: ranges
				.scheme
				.map(|r| unsafe { Scheme::new_unchecked(&bytes[r]) }),
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

	#[inline]
	pub fn as_uri(&self) -> Option<&Uri> {
		if self.scheme().is_some() {
			Some(unsafe { Uri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(std::str::from_utf8_unchecked(&self.0)) })
		} else {
			None
		}
	}

	#[inline]
	pub fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(std::str::from_utf8_unchecked(&self.0)) }
	}

	/// Returns the scheme of the URI reference, if any.
	#[inline]
	pub fn scheme(&self) -> Option<&Scheme> {
		RiRefImpl::scheme_opt(self)
	}

	/// Returns the authority part of the URI reference, if any.
	pub fn authority(&self) -> Option<&Authority> {
		RiRefImpl::authority(self)
	}

	/// Returns the path of the URI reference.
	pub fn path(&self) -> &Path {
		RiRefImpl::path(self)
	}

	pub fn query(&self) -> Option<&Query> {
		RiRefImpl::query(self)
	}

	pub fn fragment(&self) -> Option<&Fragment> {
		RiRefImpl::fragment(self)
	}

	/// Resolve the URI reference against the given *base URI*.
	///
	/// Return the resolved URI.
	/// See the [`UriRefBuf::resolve`] method for more information about the resolution process.
	#[inline]
	pub fn resolved(&self, base_iri: &(impl ?Sized + AsRef<Uri>)) -> UriBuf {
		let iri_ref = self.to_owned();
		iri_ref.into_resolved(base_iri)
	}

	/// Get this URI reference relatively to the given one.
	///
	/// # Example
	/// ```
	/// # use iref::UriRef;
	/// let a = UriRef::new(b"https://crates.io/").unwrap();
	/// let b = UriRef::new(b"https://crates.io/crates/iref").unwrap();
	/// let c = UriRef::new(b"https://crates.io/crates/json-ld").unwrap();
	/// assert_eq!(b.relative_to(a), "crates/iref");
	/// assert_eq!(a.relative_to(b), "..");
	/// assert_eq!(b.relative_to(c), "iref");
	/// assert_eq!(c.relative_to(b), "json-ld");
	/// ```
	pub fn relative_to(&self, other: &(impl ?Sized + AsRef<UriRef>)) -> UriRefBuf {
		RiRefImpl::relative_to(self, other.as_ref())
	}

	/// Get the suffix of this URI reference, if any, with regard to the given prefix URI reference..
	///
	/// Returns `Some((suffix, query, fragment))` if this URI reference is of the form
	/// `prefix/suffix?query#fragment` where `prefix` is given as parameter.
	/// Returns `None` otherwise.
	/// If the `suffix` scheme or authority is different from this path, it will return `None`.
	///
	/// See [`Path::suffix`] for more details.
	#[inline]
	pub fn suffix(
		&self,
		prefix: &(impl ?Sized + AsRef<UriRef>),
	) -> Option<(PathBuf, Option<&Query>, Option<&Fragment>)> {
		RiRefImpl::suffix(self, prefix.as_ref())
	}

	/// The URI reference without the file name, query and fragment.
	///
	/// # Example
	/// ```
	/// # use iref::UriRef;
	/// let a = UriRef::new(b"https://crates.io/crates/iref?query#fragment").unwrap();
	/// let b = UriRef::new(b"https://crates.io/crates/iref/?query#fragment").unwrap();
	/// assert_eq!(a.base(), b"https://crates.io/crates/");
	/// assert_eq!(b.base(), b"https://crates.io/crates/iref/")
	/// ```
	#[inline]
	pub fn base(&self) -> &Self {
		unsafe { Self::new_unchecked(RiRefImpl::base(self)) }
	}
}

impl AsRef<IriRef> for UriRef {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl<'a> From<&'a UriRef> for &'a IriRef {
	fn from(value: &'a UriRef) -> Self {
		value.as_iri_ref()
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Uri {
	type Error = InvalidUri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Iri {
	type Error = InvalidIri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_iri().ok_or(InvalidIri(value))
	}
}

bytestr_eq!(UriRef);

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

impl PartialEq<UriRefBuf> for UriRef {
	fn eq(&self, other: &UriRefBuf) -> bool {
		*self == *other.as_uri_ref()
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

impl PartialEq<UriBuf> for UriRef {
	fn eq(&self, other: &UriBuf) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl Eq for UriRef {}

impl PartialOrd for UriRef {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a UriRef> for UriRef {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<UriRefBuf> for UriRef {
	fn partial_cmp(&self, other: &UriRefBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl PartialOrd<Uri> for UriRef {
	fn partial_cmp(&self, other: &Uri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl<'a> PartialOrd<&'a Uri> for UriRef {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl PartialOrd<UriBuf> for UriRef {
	fn partial_cmp(&self, other: &UriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl Ord for UriRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for UriRef {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl RiRefImpl for UriRefBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = Self;

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl RiRefBufImpl for UriRefBuf {
	type Ri = Uri;
	type RiBuf = UriBuf;

	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
		Self::new_unchecked(bytes)
	}

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.0
	}

	fn into_bytes(self) -> Vec<u8> {
		self.0
	}
}

impl UriRefBuf {
	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the URI reference.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid URI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.0
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(String::from_utf8_unchecked(self.0)) }
	}

	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(UriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidUri(self))
		}
	}

	pub fn try_into_iri(self) -> Result<IriBuf, InvalidIri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(IriBuf::new_unchecked(String::from_utf8_unchecked(self.0))) }
		} else {
			Err(InvalidIri(self))
		}
	}

	pub fn path_mut(&mut self) -> PathMut {
		PathMut::from_impl(RiRefBufImpl::path_mut(self))
	}

	pub fn authority_mut(&mut self) -> Option<AuthorityMut> {
		RiRefBufImpl::authority_mut(self).map(AuthorityMut::from_impl)
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
	/// use iref::{UriRefBuf, uri::Scheme};
	///
	/// let mut a = UriRefBuf::new(b"foo/bar".to_vec()).unwrap();
	/// a.set_scheme(Some(Scheme::new(b"http").unwrap()));
	/// assert_eq!(a, "http:foo/bar");
	///
	/// let mut b = UriRefBuf::new(b"scheme://example.org/foo/bar".to_vec()).unwrap();
	/// b.set_scheme(None);
	/// assert_eq!(b, "//example.org/foo/bar");
	///
	/// let mut c = UriRefBuf::new(b"scheme:foo:bar".to_vec()).unwrap();
	/// c.set_scheme(None);
	/// assert_eq!(c, "./foo:bar");
	/// ```
	pub fn set_scheme(&mut self, scheme: Option<&Scheme>) {
		RiRefBufImpl::set_scheme(self, scheme)
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
	/// let mut a = UriRefBuf::new(b"scheme:/path".to_vec()).unwrap();
	/// a.set_authority(Some(Authority::new(b"example.org").unwrap()));
	/// assert_eq!(a, b"scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = UriRefBuf::new(b"scheme:path".to_vec()).unwrap();
	/// b.set_authority(Some(Authority::new(b"example.org").unwrap()));
	/// assert_eq!(b, b"scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = UriRefBuf::new(b"scheme://example.org//path".to_vec()).unwrap();
	/// c.set_authority(None);
	/// assert_eq!(c, b"scheme:/.//path");
	/// ```
	pub fn set_authority(&mut self, authority: Option<&Authority>) {
		RiRefBufImpl::set_authority(self, authority)
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
	/// let mut a = UriRefBuf::new(b"http://example.org/old/path".to_vec()).unwrap();
	/// a.set_path(Path::new(b"/foo/bar").unwrap());
	/// assert_eq!(a, b"http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = UriRefBuf::new(b"http://example.org/old/path".to_vec()).unwrap();
	/// b.set_path(Path::new(b"relative/path").unwrap());
	/// assert_eq!(b, b"http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = UriRefBuf::new(b"http:old/path".to_vec()).unwrap();
	/// c.set_path(Path::new(b"//foo/bar").unwrap());
	/// assert_eq!(c, b"http:/.//foo/bar");
	///
	/// // If there is no authority nor scheme, and the path beginning looks
	/// // like a scheme, it is prefixed with `./` to avoid being confused with
	/// // a scheme.
	/// let mut d = UriRefBuf::new(b"old/path".to_vec()).unwrap();
	/// d.set_path(Path::new(b"foo:bar").unwrap());
	/// assert_eq!(d, b"./foo:bar");
	/// ```
	pub fn set_path(&mut self, path: &Path) {
		RiRefBufImpl::set_path(self, path)
	}

	/// Sets the query part.
	pub fn set_query(&mut self, query: Option<&Query>) {
		RiRefBufImpl::set_query(self, query)
	}

	/// Sets the fragment part.
	pub fn set_fragment(&mut self, fragment: Option<&Fragment>) {
		RiRefBufImpl::set_fragment(self, fragment)
	}

	/// Resolve the URI reference.
	///
	/// ## Abnormal use of dot segments.
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn resolve(&mut self, base_iri: &(impl ?Sized + AsRef<Uri>)) {
		RiRefBufImpl::resolve(self, base_iri.as_ref())
	}

	pub fn into_resolved(self, base_iri: &(impl ?Sized + AsRef<Uri>)) -> UriBuf {
		RiRefBufImpl::into_resolved(self, base_iri.as_ref())
	}
}

impl AsRef<IriRef> for UriRefBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl From<UriRefBuf> for IriRefBuf {
	fn from(value: UriRefBuf) -> Self {
		value.into_iri_ref()
	}
}

impl TryFrom<UriRefBuf> for UriBuf {
	type Error = InvalidUri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

impl TryFrom<UriRefBuf> for IriBuf {
	type Error = InvalidIri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_iri()
	}
}

bytestr_eq!(UriRefBuf);

impl PartialEq<Uri> for UriRefBuf {
	fn eq(&self, other: &Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl<'a> PartialEq<&'a Uri> for UriRefBuf {
	fn eq(&self, other: &&'a Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl PartialEq<UriBuf> for UriRefBuf {
	fn eq(&self, other: &UriBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl PartialOrd<Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &Uri) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

impl<'a> PartialOrd<&'a Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

impl PartialOrd<UriBuf> for UriRefBuf {
	fn partial_cmp(&self, other: &UriBuf) -> Option<std::cmp::Ordering> {
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
	); 36] = [
		// 0 components.
		(b"", (None, None, b"", None, None)),
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
}
