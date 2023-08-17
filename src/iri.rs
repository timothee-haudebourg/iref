use std::{
	borrow::{Borrow, Cow},
	hash::{self, Hash},
};

use static_regular_grammar::RegularGrammar;

mod authority;
mod authority_mut;
mod fragment;
mod path;
mod path_mut;
mod query;
mod reference;

pub use authority::*;
pub use authority_mut::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use reference::*;

use crate::{
	common::{parse, str_eq, RiBufImpl, RiImpl, RiRefBufImpl, RiRefImpl},
	uri::InvalidUriRef,
	InvalidUri, Uri, UriBuf, UriRef, UriRefBuf,
};

macro_rules! iri_error {
	($($(#[$meta:meta])* $variant:ident : $ident:ident),*) => {
		#[derive(Debug, thiserror::Error)]
		pub enum IriError<T> {
			$(
				$(#[$meta])*
				$variant(#[from] $ident<T>)
			),*
		}

		$(
			impl<'a> From<$ident<String>> for IriError<Cow<'a, str>> {
				fn from($ident(value): $ident<String>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a str>> for IriError<Cow<'a, str>> {
				fn from($ident(value): $ident<&'a str>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}

			impl<'a> From<$ident<Vec<u8>>> for IriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<Vec<u8>>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a [u8]>> for IriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<&'a [u8]>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}
		)*
	};
}

iri_error! {
	#[error("invalid IRI: {0}")]
	Iri: InvalidIri,

	#[error("invalid IRI reference: {0}")]
	Reference: InvalidIriRef,

	#[error("invalid IRI scheme: {0}")]
	Scheme: InvalidScheme,

	#[error("invalid IRI authority: {0}")]
	Authority: InvalidAuthority,

	#[error("invalid IRI authority user info: {0}")]
	UserInfo: InvalidUserInfo,

	#[error("invalid IRI authority host: {0}")]
	Host: InvalidHost,

	#[error("invalid IRI authority port: {0}")]
	Port: InvalidPort,

	#[error("invalid IRI path: {0}")]
	Path: InvalidPath,

	#[error("invalid IRI path segment: {0}")]
	PathSegment: InvalidSegment,

	#[error("invalid IRI query: {0}")]
	Query: InvalidQuery,

	#[error("invalid IRI fragment: {0}")]
	Fragment: InvalidFragment
}

/// Internationalized Resource Identifier (IRI).
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
#[grammar(sized(IriBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Iri(str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl RiRefImpl for Iri {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = IriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiImpl for Iri {}

impl Iri {
	pub fn parts(&self) -> IriParts {
		let bytes = self.as_bytes();
		let ranges = parse::parts(bytes, 0);

		IriParts {
			scheme: unsafe { Scheme::new_unchecked(&bytes[ranges.scheme]) },
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

	/// Converts this IRI into an IRI reference.
	///
	/// All IRI are valid IRI references.
	pub fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(&self.0) }
	}

	/// Converts this IRI into an URI, if possible.
	pub fn as_uri(&self) -> Option<&Uri> {
		Uri::new(self.as_bytes()).ok()
	}

	/// Converts this IRI into an URI reference, if possible.
	pub fn as_uri_ref(&self) -> Option<&UriRef> {
		UriRef::new(self.as_bytes()).ok()
	}

	/// Returns the scheme of the IRI.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		RiImpl::scheme(self)
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
}

impl AsRef<IriRef> for Iri {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl Borrow<IriRef> for Iri {
	fn borrow(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl<'a> From<&'a Iri> for &'a IriRef {
	fn from(value: &'a Iri) -> Self {
		value.as_iri_ref()
	}
}

impl<'a> TryFrom<&'a Iri> for &'a Uri {
	type Error = InvalidUri<&'a Iri>;

	fn try_from(value: &'a Iri) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a Iri> for &'a UriRef {
	type Error = InvalidUriRef<&'a Iri>;

	fn try_from(value: &'a Iri) -> Result<Self, Self::Error> {
		value.as_uri_ref().ok_or(InvalidUriRef(value))
	}
}

str_eq!(Iri);

impl PartialEq for Iri {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a Iri> for Iri {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl PartialEq<IriBuf> for Iri {
	fn eq(&self, other: &IriBuf) -> bool {
		*self == *other.as_iri()
	}
}

impl PartialEq<IriRef> for Iri {
	fn eq(&self, other: &IriRef) -> bool {
		*self.as_iri_ref() == *other
	}
}

impl<'a> PartialEq<&'a IriRef> for Iri {
	fn eq(&self, other: &&'a IriRef) -> bool {
		*self.as_iri_ref() == **other
	}
}

impl PartialEq<IriRefBuf> for Iri {
	fn eq(&self, other: &IriRefBuf) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl Eq for Iri {}

impl PartialOrd for Iri {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a Iri> for Iri {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<IriBuf> for Iri {
	fn partial_cmp(&self, other: &IriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri())
	}
}

impl PartialOrd<IriRef> for Iri {
	fn partial_cmp(&self, other: &IriRef) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<&'a IriRef> for Iri {
	fn partial_cmp(&self, other: &&'a IriRef) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(*other)
	}
}

impl PartialOrd<IriRefBuf> for Iri {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl Ord for Iri {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for Iri {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl RiRefImpl for IriBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = IriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiImpl for IriBuf {}

impl RiRefBufImpl for IriBuf {
	type Ri = Iri;
	type RiBuf = Self;

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

impl RiBufImpl for IriBuf {}

impl IriBuf {
	/// Creates a new IRI from a byte string.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<Self, InvalidIri<Vec<u8>>> {
		match String::from_utf8(buffer) {
			Ok(string) => Self::new(string).map_err(|InvalidIri(s)| InvalidIri(s.into_bytes())),
			Err(e) => Err(InvalidIri(e.into_bytes())),
		}
	}

	/// Creates a new IRI from its scheme.
	pub fn from_scheme(scheme: SchemeBuf) -> Self {
		RiBufImpl::from_scheme(scheme)
	}

	/// Converts this IRI into an IRI reference.
	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(self.0) }
	}

	/// Converts this IRI into an URI, if possible.
	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<IriBuf>> {
		UriBuf::new(self.into_bytes()).map_err(|InvalidUri(bytes)| unsafe {
			InvalidUri(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Converts this IRI into an URI reference, if possible.
	pub fn try_into_uri_ref(self) -> Result<UriRefBuf, InvalidUriRef<IriBuf>> {
		UriRefBuf::new(self.into_bytes()).map_err(|InvalidUriRef(bytes)| unsafe {
			InvalidUriRef(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the IRI.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid IRI.
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
	/// # Example
	///
	/// ```
	/// use iref::{IriBuf, iri::Scheme};
	///
	/// let mut a = IriBuf::new("http://example.org/path".to_string()).unwrap();
	/// a.set_scheme(Scheme::new(b"https").unwrap());
	/// assert_eq!(a, "https://example.org/path");
	/// ```
	pub fn set_scheme(&mut self, new_scheme: &Scheme) {
		RiBufImpl::set_scheme(self, new_scheme)
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
	/// use iref::{IriBuf, iri::Authority};
	///
	/// let mut a = IriBuf::new("scheme:/path".to_string()).unwrap();
	/// a.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(a, "scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = IriBuf::new("scheme:path".to_string()).unwrap();
	/// b.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(b, "scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = IriBuf::new("scheme://example.org//path".to_string()).unwrap();
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
	/// use iref::{IriBuf, iri::Path};
	///
	/// let mut a = IriBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// a.set_path(Path::new("/foo/bar").unwrap());
	/// assert_eq!(a, "http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = IriBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// b.set_path(Path::new("relative/path").unwrap());
	/// assert_eq!(b, "http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = IriBuf::new("http:old/path".to_string()).unwrap();
	/// c.set_path(Path::new("//foo/bar").unwrap());
	/// assert_eq!(c, "http:/.//foo/bar");
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
}

impl AsRef<IriRef> for IriBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl Borrow<IriRef> for IriBuf {
	fn borrow(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl From<IriBuf> for IriRefBuf {
	fn from(value: IriBuf) -> Self {
		value.into_iri_ref()
	}
}

impl TryFrom<IriBuf> for UriBuf {
	type Error = InvalidUri<IriBuf>;

	fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

impl TryFrom<IriBuf> for UriRefBuf {
	type Error = InvalidUriRef<IriBuf>;

	fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
		value.try_into_uri_ref()
	}
}

str_eq!(IriBuf);

impl PartialEq<IriRef> for IriBuf {
	fn eq(&self, other: &IriRef) -> bool {
		*self.as_iri_ref() == *other
	}
}

impl<'a> PartialEq<&'a IriRef> for IriBuf {
	fn eq(&self, other: &&'a IriRef) -> bool {
		*self.as_iri_ref() == **other
	}
}

impl PartialEq<IriRefBuf> for IriBuf {
	fn eq(&self, other: &IriRefBuf) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl PartialOrd<IriRef> for IriBuf {
	fn partial_cmp(&self, other: &IriRef) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<&'a IriRef> for IriBuf {
	fn partial_cmp(&self, other: &&'a IriRef) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(*other)
	}
}

impl PartialOrd<IriRefBuf> for IriBuf {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
	}
}
