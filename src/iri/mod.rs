mod authority;
mod buffer;
mod fragment;
mod host;
mod path;
mod port;
mod query;
mod scheme;
mod segment;
mod userinfo;

use crate::{IriRef, IriRefBuf};
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

pub use self::authority::*;
pub use self::buffer::*;
pub use self::fragment::*;
pub use self::host::*;
pub use self::path::*;
pub use self::port::*;
pub use self::query::*;
pub use self::scheme::*;
pub use self::segment::*;
pub use self::userinfo::*;

/// Parsing errors.
///
/// These are the different errors raised when some part of an IRI or IRI reference has an
/// invalid syntax or encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
	/// The input data is not a valid UTF-8 encoded string.
	InvalidEncoding,

	/// The IRI part support percent-encoding, but the input data as an invalid percent-encoded
	/// character.
	/// This can occur for instance while trying to parse a query with the invalid percent encoded
	/// character `%9a`: `Query::try_from("Hello Error %9a")`.
	InvalidPercentEncoding,

	/// Occurs when one is trying to convert an [`IriRef`] with no scheme into an [`Iri`],
	/// or when an IRI is parsed with no scheme.
	MissingScheme,

	/// Occurs when the parsed [`Scheme`] is not syntactically valid.
	/// Note that even in an IRI, only ASCII letters, digit and symbols `+`, `-` and `.` are
	/// allowed.
	InvalidScheme,

	/// Occurs when the parsed [`Authority`] is not syntactically valid.
	InvalidAuthority,

	/// Occurs when the parsed [`UserInfo`] part of an [`Authority`] is not syntactically valid.
	/// Note that the userinfo part cannot include the `@` character.
	InvalidUserInfo,

	/// Occurs when the parsed [`Host`] part of an [`Authority`] is not syntactically valid.
	/// Note that the host part cannot include the `:` character.
	InvalidHost,

	/// Occurs when the parsed [`Port`] part of an [`Authority`] is not syntactically valid.
	/// This part may only contain ASCII digits.
	InvalidPort,

	/// Occurs when a path [`Segment`] is not syntactically valid.
	/// A [`Path`] segment cannot contain any `/` except at the end to denote "open" segments.
	InvalidSegment,

	/// Occurs when a [`Path`] is not syntactically valid.
	/// A path cannot contain the characters `?` and `#` delimitating the [`Query`] and
	/// [`Fragment`] parts.
	InvalidPath,

	/// Occurs when a [`Query`] part is not syntactically valid.
	InvalidQuery,

	/// Occurs when a [`Fragment`] part is not syntactically valid.
	InvalidFragment,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Error::InvalidEncoding => "Invalid encoding",
			Error::InvalidPercentEncoding => "Invalid percent encoding",
			Error::MissingScheme => "Missing scheme",
			Error::InvalidScheme => "Invalid scheme",
			Error::InvalidAuthority => "Invalid authority",
			Error::InvalidUserInfo => "Invalid user info",
			Error::InvalidHost => "Invalid host",
			Error::InvalidPort => "Invalid port",
			Error::InvalidSegment => "Invalid segment",
			Error::InvalidPath => "Invalid path",
			Error::InvalidQuery => "Invalid query",
			Error::InvalidFragment => "Invalid fragment",
		})
	}
}

impl StdError for Error {}

/// IRI slice.
///
/// Wrapper around a borrowed bytes slice representing an IRI.
/// An IRI can be seen as an IRI-reference with a defined [`Scheme`].
/// All methods of [`IriRef`] are available from this type, however the [`scheme`](Iri::scheme) method
/// is redefined to always return some scheme.
///
/// ## Example
///
/// ```rust
/// # extern crate iref;
/// # use iref::Iri;
/// # fn main() -> Result<(), iref::Error> {
/// let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#frag")?;
///
/// println!("scheme: {}", iri.scheme()); // note the absence of `unwrap` here since
///                                       // the scheme is always defined in an IRI.
/// println!("authority: {}", iri.authority().unwrap());
/// println!("path: {}", iri.path());
/// println!("query: {}", iri.query().unwrap());
/// println!("fragment: {}", iri.fragment().unwrap());
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Iri<'a>(IriRef<'a>);

impl<'a> Iri<'a> {
	/// Create a new IRI slice from a bytes slice.
	///
	/// This may fail if the source slice is not UTF-8 encoded, or is not a valid IRI.
	#[inline]
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		let iri_ref = IriRef::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(Iri(iri_ref))
		} else {
			Err(Error::MissingScheme)
		}
	}

	/// Create a new IRI from a string.
	///
	/// This replaces a [`std::str::FromStr`] implementation as the trait is
	/// incompatiple with the result storing the input which [`Iri`] does.
	#[allow(clippy::should_implement_trait)]
	pub fn from_str(s: &'a str) -> Result<Self, Error> {
		Self::new(s)
	}

	/// Convert the slice-like [`Iri`] into the owned version [`IriBuf`].
	pub fn to_owned(self) -> IriBuf {
		IriBuf(self.0.to_owned())
	}

	/// Build an IRI from an IRI reference.
	#[inline]
	pub const fn from_iri_ref(iri_ref: IriRef<'a>) -> Iri<'a> {
		Iri(iri_ref)
	}

	/// Get an [`IriRef`] out of this IRI.
	///
	/// An IRI is always a valid IRI-reference.
	#[inline]
	pub fn as_iri_ref(&self) -> IriRef<'a> {
		self.0
	}

	/// Get the scheme of the IRI.
	///
	/// Contrarily to [`IriRef`], the scheme of an IRI is always defined.
	#[inline]
	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}
}

impl<'a> Deref for Iri<'a> {
	type Target = IriRef<'a>;

	#[inline]
	fn deref(&self) -> &IriRef<'a> {
		&self.0
	}
}

impl<'a> fmt::Display for Iri<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl<'a> fmt::Debug for Iri<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl<'a> PartialEq for Iri<'a> {
	#[inline]
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> Eq for Iri<'a> {}

impl<'a> PartialEq<IriRef<'a>> for Iri<'a> {
	#[inline]
	fn eq(&self, other: &IriRef<'a>) -> bool {
		self.as_iri_ref() == *other
	}
}

impl<'a> PartialEq<IriRefBuf> for Iri<'a> {
	#[inline]
	fn eq(&self, other: &IriRefBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<IriBuf> for Iri<'a> {
	#[inline]
	fn eq(&self, other: &IriBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a str> for Iri<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_iri_ref().eq(other)
	}
}

impl<'a> PartialOrd for Iri<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> Ord for Iri<'a> {
	#[inline]
	fn cmp(&self, other: &Iri<'a>) -> Ordering {
		self.as_iri_ref().cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriRef<'a>> for Iri<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<IriRefBuf> for Iri<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriBuf> for Iri<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<&'a IriBuf> for Iri<'a> {
	#[inline]
	fn from(buffer: &'a IriBuf) -> Iri<'a> {
		buffer.as_iri()
	}
}

impl<'a> TryFrom<IriRef<'a>> for Iri<'a> {
	type Error = IriRef<'a>;

	#[inline]
	fn try_from(iri_ref: IriRef<'a>) -> Result<Iri<'a>, IriRef<'a>> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(Iri(iri_ref))
		} else {
			Err(iri_ref)
		}
	}
}

impl<'a> TryFrom<&'a IriRefBuf> for Iri<'a> {
	type Error = Error;

	#[inline]
	fn try_from(buffer: &'a IriRefBuf) -> Result<Iri<'a>, Error> {
		if buffer.p.scheme_len.is_some() {
			Ok(Iri(buffer.as_iri_ref()))
		} else {
			Err(Error::InvalidScheme)
		}
	}
}

impl<'a> Hash for Iri<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}

#[cfg(feature = "hashbrown")]
impl<'a> hashbrown::Equivalent<IriRefBuf> for Iri<'a> {
	fn equivalent(&self, key: &IriRefBuf) -> bool {
		*self == *key
	}
}

#[cfg(feature = "hashbrown")]
impl<'a> hashbrown::Equivalent<IriBuf> for Iri<'a> {
	fn equivalent(&self, key: &IriBuf) -> bool {
		*self == *key
	}
}

#[cfg(feature = "hashbrown")]
impl<'a, 'b> hashbrown::Equivalent<IriRef<'b>> for Iri<'a> {
	fn equivalent(&self, key: &IriRef<'b>) -> bool {
		*self == *key
	}
}

impl<'a> AsIri for Iri<'a> {
	#[inline]
	fn as_iri(&self) -> Iri {
		*self
	}
}

impl<'a> AsIriRef for Iri<'a> {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		self.as_iri_ref()
	}
}

/// Cheap reference-to-IRI-reference convertion.
///
/// This is to be used instead of `AsRef<IriRef>` until custom DSTs are introduced.
pub trait AsIriRef {
	fn as_iri_ref(&self) -> IriRef;
}

/// Cheap reference-to-IRI convertion.
///
/// This is to be used instead of `AsRef<Iri>` until custom DSTs are introduced.
pub trait AsIri {
	fn as_iri(&self) -> Iri;
}

impl<'a, T: AsIri> AsIri for &'a T {
	#[inline]
	fn as_iri(&self) -> Iri {
		(*self).as_iri()
	}
}

impl<'a, T: AsIriRef> AsIriRef for &'a T {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		(*self).as_iri_ref()
	}
}

impl<'a> AsRef<str> for Iri<'a> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<'a> AsRef<[u8]> for Iri<'a> {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> Borrow<str> for Iri<'a> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<'a> Borrow<[u8]> for Iri<'a> {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}
