use std::hash::{self, Hash};

use static_regular_grammar::RegularGrammar;

use crate::{
	common::{parse, str_eq, RiRefBufImpl, RiRefImpl},
	uri::InvalidUriRef,
	InvalidIri, InvalidUri, Iri, IriBuf, Uri, UriBuf, UriRef, UriRefBuf,
};

use super::{Authority, AuthorityMut, Fragment, Path, PathBuf, PathMut, Query, Scheme};

/// IRI reference.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI-reference",
	cache = "automata/iri/reference.aut.cbor"
)]
#[grammar(sized(
	IriRefBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct IriRef(str);

impl RiRefImpl for IriRef {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = IriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IriRefParts<'a> {
	pub scheme: Option<&'a Scheme>,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl IriRef {
	pub fn parts(&self) -> IriRefParts {
		let bytes = self.as_bytes();
		let ranges = parse::reference_parts(bytes, 0);

		IriRefParts {
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

	/// Converts this IRI reference into an IRI, if possible.
	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	/// Converts this IRI reference into an URI, if possible.
	pub fn as_uri(&self) -> Option<&Uri> {
		Uri::new(self.as_bytes()).ok()
	}

	/// Converts this IRI reference into an URI reference, if possible.
	pub fn as_uri_ref(&self) -> Option<&UriRef> {
		UriRef::new(self.as_bytes()).ok()
	}

	/// Returns the scheme of the IRI reference, if any.
	#[inline]
	pub fn scheme(&self) -> Option<&Scheme> {
		RiRefImpl::scheme_opt(self)
	}

	/// Returns the authority part of the IRI reference, if any.
	pub fn authority(&self) -> Option<&Authority> {
		RiRefImpl::authority(self)
	}

	/// Returns the path of the IRI reference.
	pub fn path(&self) -> &Path {
		RiRefImpl::path(self)
	}

	pub fn query(&self) -> Option<&Query> {
		RiRefImpl::query(self)
	}

	pub fn fragment(&self) -> Option<&Fragment> {
		RiRefImpl::fragment(self)
	}

	/// Resolve the IRI reference against the given *base IRI*.
	///
	/// Return the resolved IRI.
	/// See the [`IriRefBuf::resolve`] method for more information about the resolution process.
	#[inline]
	pub fn resolved(&self, base_iri: &(impl ?Sized + AsRef<Iri>)) -> IriBuf {
		let iri_ref = self.to_owned();
		iri_ref.into_resolved(base_iri)
	}

	/// Get this IRI reference relatively to the given one.
	///
	/// # Example
	/// ```
	/// # use iref::IriRef;
	/// let a = IriRef::new("https://crates.io/").unwrap();
	/// let b = IriRef::new("https://crates.io/crates/iref").unwrap();
	/// let c = IriRef::new("https://crates.io/crates/json-ld").unwrap();
	/// assert_eq!(b.relative_to(a), "crates/iref");
	/// assert_eq!(a.relative_to(b), "..");
	/// assert_eq!(b.relative_to(c), "iref");
	/// assert_eq!(c.relative_to(b), "json-ld");
	/// ```
	pub fn relative_to(&self, other: &(impl ?Sized + AsRef<IriRef>)) -> IriRefBuf {
		RiRefImpl::relative_to(self, other.as_ref())
	}

	/// Get the suffix of this IRI reference, if any, with regard to the given prefix IRI reference..
	///
	/// Returns `Some((suffix, query, fragment))` if this IRI reference is of the form
	/// `prefix/suffix?query#fragment` where `prefix` is given as parameter.
	/// Returns `None` otherwise.
	/// If the `suffix` scheme or authority is different from this path, it will return `None`.
	///
	/// See [`Path::suffix`] for more details.
	#[inline]
	pub fn suffix(
		&self,
		prefix: &(impl ?Sized + AsRef<IriRef>),
	) -> Option<(PathBuf, Option<&Query>, Option<&Fragment>)> {
		RiRefImpl::suffix(self, prefix.as_ref())
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
		unsafe { Self::new_unchecked(std::str::from_utf8_unchecked(RiRefImpl::base(self))) }
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a Iri {
	type Error = InvalidIri<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_iri().ok_or(InvalidIri(value))
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a Uri {
	type Error = InvalidUri<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a UriRef {
	type Error = InvalidUriRef<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_uri_ref().ok_or(InvalidUriRef(value))
	}
}

str_eq!(IriRef);

impl PartialEq for IriRef {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a IriRef> for IriRef {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl PartialEq<IriRefBuf> for IriRef {
	fn eq(&self, other: &IriRefBuf) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl PartialEq<Iri> for IriRef {
	fn eq(&self, other: &Iri) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a Iri> for IriRef {
	fn eq(&self, other: &&'a Iri) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl PartialEq<IriBuf> for IriRef {
	fn eq(&self, other: &IriBuf) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl Eq for IriRef {}

impl PartialOrd for IriRef {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a IriRef> for IriRef {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<IriRefBuf> for IriRef {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl PartialOrd<Iri> for IriRef {
	fn partial_cmp(&self, other: &Iri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl<'a> PartialOrd<&'a Iri> for IriRef {
	fn partial_cmp(&self, other: &&'a Iri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl PartialOrd<IriBuf> for IriRef {
	fn partial_cmp(&self, other: &IriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl Ord for IriRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for IriRef {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl RiRefImpl for IriRefBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = Self;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiRefBufImpl for IriRefBuf {
	type Ri = Iri;
	type RiBuf = IriBuf;

	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
		Self::new_unchecked(String::from_utf8_unchecked(bytes))
	}

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	fn into_bytes(self) -> Vec<u8> {
		self.0.into_bytes()
	}
}

impl IriRefBuf {
	/// Creates a new IRI reference from a byte string.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<Self, InvalidIriRef<Vec<u8>>> {
		match String::from_utf8(buffer) {
			Ok(string) => {
				Self::new(string).map_err(|InvalidIriRef(s)| InvalidIriRef(s.into_bytes()))
			}
			Err(e) => Err(InvalidIriRef(e.into_bytes())),
		}
	}

	/// Creates a new IRI reference from a byte string without varidation.
	///
	/// # Safety
	///
	/// The input bytes must be a valid IRI reference.
	#[inline]
	pub unsafe fn from_vec_unchecked(buffer: Vec<u8>) -> Self {
		Self::new_unchecked(String::from_utf8_unchecked(buffer))
	}

	/// Converts this IRI reference into an IRI, if possible.
	pub fn try_into_iri(self) -> Result<IriBuf, InvalidIri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(IriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidIri(self))
		}
	}

	/// Converts this IRI reference into an URI, if possible.
	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<Self>> {
		UriBuf::new(self.into_bytes()).map_err(|InvalidUri(bytes)| unsafe {
			InvalidUri(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Converts this IRI reference into an URI reference, if possible.
	pub fn try_into_uri_ref(self) -> Result<UriRefBuf, InvalidUriRef<Self>> {
		UriRefBuf::new(self.into_bytes()).map_err(|InvalidUriRef(bytes)| unsafe {
			InvalidUriRef(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Returns the underlying bytes representing the IRI reference as a mutable
	/// `Vec<u8>`.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid IRI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
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

	/// Resolve the IRI reference.
	///
	/// ## Abnormal use of dot segments.
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn resolve(&mut self, base_iri: &(impl ?Sized + AsRef<Iri>)) {
		RiRefBufImpl::resolve(self, base_iri.as_ref())
	}

	pub fn into_resolved(self, base_iri: &(impl ?Sized + AsRef<Iri>)) -> IriBuf {
		RiRefBufImpl::into_resolved(self, base_iri.as_ref())
	}
}

impl TryFrom<IriRefBuf> for IriBuf {
	type Error = InvalidIri<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_iri()
	}
}

impl TryFrom<IriRefBuf> for UriBuf {
	type Error = InvalidUri<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

impl TryFrom<IriRefBuf> for UriRefBuf {
	type Error = InvalidUriRef<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri_ref()
	}
}

str_eq!(IriRefBuf);

impl PartialEq<Iri> for IriRefBuf {
	fn eq(&self, other: &Iri) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a Iri> for IriRefBuf {
	fn eq(&self, other: &&'a Iri) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl PartialEq<IriBuf> for IriRefBuf {
	fn eq(&self, other: &IriBuf) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl PartialOrd<Iri> for IriRefBuf {
	fn partial_cmp(&self, other: &Iri) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
	}
}

impl<'a> PartialOrd<&'a Iri> for IriRefBuf {
	fn partial_cmp(&self, other: &&'a Iri) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
	}
}

impl PartialOrd<IriBuf> for IriRefBuf {
	fn partial_cmp(&self, other: &IriBuf) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
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
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.authority().map(Authority::as_str), expected.1)
		}
	}

	#[test]
	fn set_authority() {
		let vectors = [
			("scheme:/path", Some("authority"), "scheme://authority/path"),
			("scheme:path", Some("authority"), "scheme://authority/path"),
			("scheme://authority//path", None, "scheme:/.//path"),
		];

		for (input, authority, expected) in vectors {
			let mut buffer = IriRefBuf::new(input.to_string()).unwrap();
			let authority = authority.map(Authority::new).transpose().unwrap();
			buffer.set_authority(authority);
			// eprintln!("{input}, {authority:?} => {buffer}, {expected}");
			assert_eq!(buffer.as_str(), expected)
		}
	}

	#[test]
	fn path() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.path().as_str(), expected.2)
		}
	}

	#[test]
	fn query() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.query().map(Query::as_str), expected.3)
		}
	}

	#[test]
	fn fragment() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.fragment().map(Fragment::as_str), expected.4)
		}
	}

	#[test]
	fn disambiguate_scheme() {
		let mut iri_ref = IriRefBuf::new("scheme:a:b/c".to_string()).unwrap();
		iri_ref.set_scheme(None);
		assert_eq!(iri_ref.as_str(), "./a:b/c")
	}

	#[test]
	fn disambiguate_authority() {
		let mut iri_ref = IriRefBuf::new("//host//path".to_string()).unwrap();
		iri_ref.set_authority(None);
		assert_eq!(iri_ref.as_str(), "/.//path")
	}

	#[test]
	fn unambiguous_resolution() {
		let base_iri = Iri::new("http:/a/b").unwrap();

		let tests = [("../..//", "http:/..//")];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_normal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
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
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_abnormal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		// NOTE we implement [Errata 4547](https://www.rfc-editor.org/errata/eid4547)
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("../../../g", "http://a/../g"), // NOTE without Errata 4547: "http://a/g"
			("../../../../g", "http://a/../../g"), // NOTE without Errata 4547: "http://a/g"
			("/./g", "http://a/g"),
			("/../g", "http://a/../g"), // NOTE without Errata 4547: "http://a/g"
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
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn more_resolutions1() {
		let base_iri = Iri::new("http://a/bb/ccc/d;p?q").unwrap();

		let tests = [
			("#s", "http://a/bb/ccc/d;p?q#s"),
			("", "http://a/bb/ccc/d;p?q"),
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions2() {
		let base_iri = Iri::new("http://a/bb/ccc/./d;p?q").unwrap();

		let tests = [
			("..", "http://a/bb/"),
			("../", "http://a/bb/"),
			("../g", "http://a/bb/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions3() {
		let base_iri = Iri::new("http://ab//de//ghi").unwrap();

		let tests = [
			("xyz", "http://ab//de//xyz"),
			("./xyz", "http://ab//de//xyz"),
			("../xyz", "http://ab//de/xyz"),
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	// https://github.com/timothee-haudebourg/iref/issues/14
	#[test]
	fn reference_resolution_with_scheme_no_disambiguation() {
		let base = Iri::new("scheme:a:b/").unwrap();
		let mut iri = IriRefBuf::new("Foo".to_string()).unwrap();
		iri.resolve(base);

		assert_eq!(iri.to_string(), "scheme:a:b/Foo")
	}

	#[test]
	fn relative_to() {
		let base =
			IriRef::new("https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld").unwrap();
		let vectors = [
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
		];

		for (input, expected) in &vectors {
			let input = IriRef::new(input).unwrap();
			assert_eq!(input.relative_to(base), *expected)
		}
	}
}
