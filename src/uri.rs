use static_regular_grammar::RegularGrammar;
use std::{
	borrow::{Borrow, Cow},
	hash::{self, Hash},
};

mod authority;
mod authority_mut;
mod fragment;
mod path;
mod path_mut;
mod query;
mod reference;
mod scheme;

pub use authority::*;
pub use authority_mut::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use reference::*;
pub use scheme::*;

use crate::{
	common::{bytestr_eq, parse, RiBufImpl, RiImpl, RiRefBufImpl, RiRefImpl},
	Iri, IriBuf, IriRef, IriRefBuf,
};

macro_rules! uri_error {
	($($(#[$meta:meta])* $variant:ident : $ident:ident),*) => {
		#[derive(Debug, thiserror::Error)]
		pub enum UriError<T> {
			$(
				$(#[$meta])*
				$variant(#[from] $ident<T>)
			),*
		}

		$(
			impl<'a> From<$ident<String>> for UriError<Cow<'a, str>> {
				fn from($ident(value): $ident<String>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a str>> for UriError<Cow<'a, str>> {
				fn from($ident(value): $ident<&'a str>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}

			impl<'a> From<$ident<Vec<u8>>> for UriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<Vec<u8>>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a [u8]>> for UriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<&'a [u8]>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}
		)*
	};
}

uri_error! {
	#[error("invalid URI: {0}")]
	Uri: InvalidUri,

	#[error("invalid URI reference: {0}")]
	Reference: InvalidUriRef,

	#[error("invalid URI scheme: {0}")]
	Scheme: InvalidScheme,

	#[error("invalid URI authority: {0}")]
	Authority: InvalidAuthority,

	#[error("invalid URI authority user info: {0}")]
	UserInfo: InvalidUserInfo,

	#[error("invalid URI authority host: {0}")]
	Host: InvalidHost,

	#[error("invalid URI authority port: {0}")]
	Port: InvalidPort,

	#[error("invalid URI path: {0}")]
	Path: InvalidPath,

	#[error("invalid URI path segment: {0}")]
	PathSegment: InvalidSegment,

	#[error("invalid URI query: {0}")]
	Query: InvalidQuery,

	#[error("invalid URI fragment: {0}")]
	Fragment: InvalidFragment
}

/// Uniform Resource Identifier (URI).
#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "URI",
	ascii,
	cache = "automata/uri.aut.cbor"
)]
#[grammar(sized(UriBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Uri([u8]);

impl RiRefImpl for Uri {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = UriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl RiImpl for Uri {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl Uri {
	pub fn parts(&self) -> UriParts {
		let bytes = self.as_bytes();
		let ranges = parse::parts(bytes, 0);

		UriParts {
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

	/// Converts this URI into an URI reference.
	///
	/// All IRI are valid URI references.
	pub fn as_uri_ref(&self) -> &UriRef {
		unsafe { UriRef::new_unchecked(&self.0) }
	}

	pub fn as_iri(&self) -> &Iri {
		unsafe { Iri::new_unchecked(std::str::from_utf8_unchecked(&self.0)) }
	}

	pub fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(std::str::from_utf8_unchecked(&self.0)) }
	}

	/// Returns the scheme of the URI.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		RiImpl::scheme(self)
	}

	/// Returns the authority part of the URI, if any.
	pub fn authority(&self) -> Option<&Authority> {
		RiRefImpl::authority(self)
	}

	/// Returns the path of the URI.
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

impl AsRef<UriRef> for Uri {
	fn as_ref(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

impl AsRef<Iri> for Uri {
	fn as_ref(&self) -> &Iri {
		self.as_iri()
	}
}

impl AsRef<IriRef> for Uri {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl Borrow<UriRef> for Uri {
	fn borrow(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

impl Borrow<Iri> for Uri {
	fn borrow(&self) -> &Iri {
		self.as_iri()
	}
}

impl Borrow<IriRef> for Uri {
	fn borrow(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

bytestr_eq!(Uri);

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

impl PartialEq<UriBuf> for Uri {
	fn eq(&self, other: &UriBuf) -> bool {
		*self == *other.as_uri()
	}
}

impl PartialEq<UriRef> for Uri {
	fn eq(&self, other: &UriRef) -> bool {
		*self.as_uri_ref() == *other
	}
}

impl<'a> PartialEq<&'a UriRef> for Uri {
	fn eq(&self, other: &&'a UriRef) -> bool {
		*self.as_uri_ref() == **other
	}
}

impl PartialEq<UriRefBuf> for Uri {
	fn eq(&self, other: &UriRefBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl Eq for Uri {}

impl PartialOrd for Uri {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a Uri> for Uri {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<UriBuf> for Uri {
	fn partial_cmp(&self, other: &UriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri())
	}
}

impl PartialOrd<UriRef> for Uri {
	fn partial_cmp(&self, other: &UriRef) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<&'a UriRef> for Uri {
	fn partial_cmp(&self, other: &&'a UriRef) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(*other)
	}
}

impl PartialOrd<UriRefBuf> for Uri {
	fn partial_cmp(&self, other: &UriRefBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl Ord for Uri {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for Uri {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl RiRefImpl for UriBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	type RiRefBuf = UriRefBuf;

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl RiImpl for UriBuf {}

impl RiRefBufImpl for UriBuf {
	type Ri = Uri;
	type RiBuf = Self;

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

impl RiBufImpl for UriBuf {}

impl UriBuf {
	pub fn from_scheme(scheme: SchemeBuf) -> Self {
		RiBufImpl::from_scheme(scheme)
	}

	pub fn into_uri_ref(self) -> UriRefBuf {
		unsafe { UriRefBuf::new_unchecked(self.0) }
	}

	pub fn into_iri(self) -> IriBuf {
		unsafe { IriBuf::new_unchecked(String::from_utf8_unchecked(self.0)) }
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(String::from_utf8_unchecked(self.0)) }
	}

	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the URI.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid URI.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.0
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
	/// use iref::{UriBuf, uri::Scheme};
	///
	/// let mut a = UriBuf::new(b"http://example.org/path".to_vec()).unwrap();
	/// a.set_scheme(Scheme::new(b"https").unwrap());
	/// assert_eq!(a, b"https://example.org/path");
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
	/// use iref::{UriBuf, uri::Authority};
	///
	/// let mut a = UriBuf::new(b"scheme:/path".to_vec()).unwrap();
	/// a.set_authority(Some(Authority::new(b"example.org").unwrap()));
	/// assert_eq!(a, b"scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = UriBuf::new(b"scheme:path".to_vec()).unwrap();
	/// b.set_authority(Some(Authority::new(b"example.org").unwrap()));
	/// assert_eq!(b, b"scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = UriBuf::new(b"scheme://example.org//path".to_vec()).unwrap();
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
	/// use iref::{UriBuf, uri::Path};
	///
	/// let mut a = UriBuf::new(b"http://example.org/old/path".to_vec()).unwrap();
	/// a.set_path(Path::new(b"/foo/bar").unwrap());
	/// assert_eq!(a, b"http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = UriBuf::new(b"http://example.org/old/path".to_vec()).unwrap();
	/// b.set_path(Path::new(b"relative/path").unwrap());
	/// assert_eq!(b, b"http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = UriBuf::new(b"http:old/path".to_vec()).unwrap();
	/// c.set_path(Path::new(b"//foo/bar").unwrap());
	/// assert_eq!(c, b"http:/.//foo/bar");
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

impl AsRef<UriRef> for UriBuf {
	fn as_ref(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

impl AsRef<Iri> for UriBuf {
	fn as_ref(&self) -> &Iri {
		self.as_iri()
	}
}

impl AsRef<IriRef> for UriBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl Borrow<UriRef> for UriBuf {
	fn borrow(&self) -> &UriRef {
		self.as_uri_ref()
	}
}

impl Borrow<Iri> for UriBuf {
	fn borrow(&self) -> &Iri {
		self.as_iri()
	}
}

impl Borrow<IriRef> for UriBuf {
	fn borrow(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl From<UriBuf> for UriRefBuf {
	fn from(value: UriBuf) -> Self {
		value.into_uri_ref()
	}
}

bytestr_eq!(UriBuf);

impl PartialEq<UriRef> for UriBuf {
	fn eq(&self, other: &UriRef) -> bool {
		*self.as_uri_ref() == *other
	}
}

impl<'a> PartialEq<&'a UriRef> for UriBuf {
	fn eq(&self, other: &&'a UriRef) -> bool {
		*self.as_uri_ref() == **other
	}
}

impl PartialEq<UriRefBuf> for UriBuf {
	fn eq(&self, other: &UriRefBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl PartialOrd<UriRef> for UriBuf {
	fn partial_cmp(&self, other: &UriRef) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<&'a UriRef> for UriBuf {
	fn partial_cmp(&self, other: &&'a UriRef) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(*other)
	}
}

impl PartialOrd<UriRefBuf> for UriBuf {
	fn partial_cmp(&self, other: &UriRefBuf) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}
