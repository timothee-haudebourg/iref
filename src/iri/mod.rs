mod scheme;
mod userinfo;
mod host;
mod port;
mod authority;
mod segment;
mod path;
mod buffer;
mod query;
mod fragment;

use std::ops::Deref;
use std::convert::TryFrom;
use std::fmt;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use crate::{IriRef, IriRefBuf};

pub use self::scheme::*;
pub use self::userinfo::*;
pub use self::host::*;
pub use self::port::*;
pub use self::authority::*;
pub use self::segment::*;
pub use self::path::*;
pub use self::buffer::*;
pub use self::query::*;
pub use self::fragment::*;

#[derive(Debug)]
pub enum Error {
	/// The input data is not a valid UTF-8 encoded string.
	InvalidEncoding,

	Invalid,

	InvalidScheme,

	InvalidAuthority,

	InvalidUserInfo,

	InvalidHost,

	InvalidPort,

	InvalidSegment,

	InvalidPath,

	InvalidQuery,

	InvalidFragment,

	InvalidPCTEncoded,

	EmptyPath
}

/// IRI slice.
///
/// Note that in future versions, this will most likely become a custom dynamic sized type,
/// similar to `str`.
#[derive(Clone, Copy)]
pub struct Iri<'a>(IriRef<'a>);

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		let iri_ref = IriRef::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(Iri(iri_ref))
		} else {
			Err(Error::Invalid)
		}
	}

	#[inline]
	pub fn as_iri_ref(&self) -> IriRef<'a> {
		self.0
	}

	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}
}

impl<'a> Deref for Iri<'a> {
	type Target = IriRef<'a>;

	fn deref(&self) -> &IriRef<'a> {
		&self.0
	}
}

impl<'a> fmt::Display for Iri<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl<'a> fmt::Debug for Iri<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl<'a> PartialEq for Iri<'a> {
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> Eq for Iri<'a> { }

impl<'a> PartialEq<IriRef<'a>> for Iri<'a> {
	fn eq(&self, other: &IriRef<'a>) -> bool {
		self.as_iri_ref() == *other
	}
}

impl<'a> PartialEq<IriRefBuf> for Iri<'a> {
	fn eq(&self, other: &IriRefBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<IriBuf> for Iri<'a> {
	fn eq(&self, other: &IriBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a str> for Iri<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_iri_ref().eq(other)
	}
}

impl<'a> PartialOrd for Iri<'a> {
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> Ord for Iri<'a> {
	fn cmp(&self, other: &Iri<'a>) -> Ordering {
		self.as_iri_ref().cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriRef<'a>> for Iri<'a> {
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<IriRefBuf> for Iri<'a> {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriBuf> for Iri<'a> {
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<&'a IriBuf> for Iri<'a> {
	fn from(buffer: &'a IriBuf) -> Iri<'a> {
		buffer.as_iri()
	}
}

impl<'a> TryFrom<IriRef<'a>> for Iri<'a> {
	type Error = IriRef<'a>;

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

	fn try_from(buffer: &'a IriRefBuf) -> Result<Iri<'a>, Error> {
		if buffer.p.scheme_len.is_some() {
			Ok(Iri(buffer.as_iri_ref()))
		} else {
			Err(Error::InvalidScheme)
		}
	}
}

impl<'a> Hash for Iri<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}
