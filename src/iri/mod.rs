pub use crate::uri::{InvalidScheme, Scheme, SchemeBuf};
use static_automata::grammar;
use std::{
	borrow::Borrow,
	hash::{self, Hash},
	ops::Deref,
};

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

use crate::{InvalidUri, Uri, UriBuf, UriRef, UriRefBuf, uri::InvalidUriRef};

#[grammar(
	file = "grammar.abnf",
	export("IRI", "IRI-reference" as IriRef, "iauthority" as Authority, "ihost" as Host, "iuserinfo" as UserInfo, "ipath" as Path, "isegment" as Segment, "iquery" as Query, "ifragment" as Fragment)
)]
mod grammar {}

crate::common::borrowed! {
	/// Internationalized Resource Identifier (IRI).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::iri::{Iri, Scheme, Authority, Path, Query, Fragment};
	/// # fn main() -> Result<(), iref::InvalidIri<&'static str>> {
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
	"IRI": iri, Iri, IriBuf, IriRef, IriRefBuf
}

#[macro_export]
macro_rules! iri {
	($value:literal) => {
		const {
			match $crate::iri::Iri::from_str($value) {
				Ok(value) => value,
				Err(_) => panic!("invalid IRI"),
			}
		}
	};
}

impl Iri {
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
}

impl Deref for Iri {
	type Target = IriRef;

	fn deref(&self) -> &Self::Target {
		self.as_iri_ref()
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

// crate::common::owned!(Iri, IriBuf, IriRef, IriRefBuf);

impl IriBuf {
	/// Creates a new IRI from a byte string.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<Self, InvalidIri<Vec<u8>>> {
		match String::from_utf8(buffer) {
			Ok(string) => Self::new(string).map_err(|InvalidIri(s)| InvalidIri(s.into_bytes())),
			Err(e) => Err(InvalidIri(e.into_bytes())),
		}
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
		unsafe { self.0.as_mut_vec() }
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
