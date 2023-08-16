use static_regular_grammar::RegularGrammar;
use std::{
	borrow::Cow,
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

use crate::common::{parse, RiBufImpl, RiImpl, RiRefBufImpl, RiRefImpl};

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

macro_rules! bytestr_eq {
	($ident:ident) => {
		impl<const N: usize> PartialEq<[u8; N]> for $ident {
			fn eq(&self, other: &[u8; N]) -> bool {
				self.as_bytes() == other
			}
		}

		impl<'a, const N: usize> PartialEq<&'a [u8; N]> for $ident {
			fn eq(&self, other: &&'a [u8; N]) -> bool {
				self.as_bytes() == *other
			}
		}

		impl PartialEq<[u8]> for $ident {
			fn eq(&self, other: &[u8]) -> bool {
				self.as_bytes() == other
			}
		}

		impl<'a> PartialEq<&'a [u8]> for $ident {
			fn eq(&self, other: &&'a [u8]) -> bool {
				self.as_bytes() == *other
			}
		}

		impl PartialEq<str> for $ident {
			fn eq(&self, other: &str) -> bool {
				self.as_str() == other
			}
		}

		impl<'a> PartialEq<&'a str> for $ident {
			fn eq(&self, other: &&'a str) -> bool {
				self.as_str() == *other
			}
		}

		impl PartialEq<String> for $ident {
			fn eq(&self, other: &String) -> bool {
				self.as_str() == other.as_str()
			}
		}
	};
}

pub(crate) use bytestr_eq;

bytestr_eq!(Uri);

impl PartialEq for Uri {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl Eq for Uri {}

impl PartialOrd for Uri {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
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

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl RiImpl for UriBuf {}

impl RiRefBufImpl for UriBuf {
	type Ri = Uri;
	type RiBuf = Self;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.0
	}

	fn into_bytes(self) -> Vec<u8> {
		self.0
	}
}

impl RiBufImpl for UriBuf {
	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
		Self::new_unchecked(bytes)
	}
}

impl UriBuf {
	pub fn from_scheme(scheme: SchemeBuf) -> Self {
		RiBufImpl::from_scheme(scheme)
	}

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

impl From<UriBuf> for UriRefBuf {
	fn from(value: UriBuf) -> Self {
		value.into_uri_ref()
	}
}

bytestr_eq!(UriBuf);
