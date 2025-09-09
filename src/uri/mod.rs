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
mod scheme;

pub use authority::*;
pub use error::*;
pub use fragment::*;
pub use path::*;
pub use query::*;
pub use reference::*;
pub use scheme::*;

#[grammar(
	file = "grammar.abnf",
	export("URI", "URI-reference" as UriRef, "scheme", "authority", "host", "port", "userinfo" as UserInfo, "path", "segment", "query", "fragment")
)]
mod grammar {}

use crate::{Iri, IriBuf, IriRef, IriRefBuf};

crate::common::borrowed! {
	/// Uniform Resource Identifier (URI).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{Uri, Scheme, Authority, Path, Query, Fragment};
	/// # fn main() -> Result<(), iref::InvalidUri<&'static str>> {
	/// let iri = Uri::new("https://www.rust-lang.org/foo/bar?query#fragment")?;
	///
	/// assert_eq!(iri.scheme(), Scheme::new(b"https").unwrap());
	/// assert_eq!(iri.authority(), Some(Authority::new("www.rust-lang.org").unwrap()));
	/// assert_eq!(iri.path(), Path::new("/foo/bar").unwrap());
	/// assert_eq!(iri.query(), Some(Query::new("query").unwrap()));
	/// assert_eq!(iri.fragment(), Some(Fragment::new("fragment").unwrap()));
	/// #
	/// # Ok(())
	/// # }
	"URI": uri, Uri, UriBuf, UriRef, UriRefBuf
}

#[macro_export]
macro_rules! uri {
	($value:literal) => {
		match $crate::uri::Uri::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI"),
		}
	};
}

impl Uri {
	/// Converts this URI into an URI reference.
	///
	/// All IRI are valid URI references.
	pub fn as_uri_ref(&self) -> &UriRef {
		unsafe { UriRef::new_unchecked(&self.0) }
	}

	pub fn as_iri(&self) -> &Iri {
		unsafe { Iri::new_unchecked(&self.0) }
	}

	pub fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(&self.0) }
	}
}

impl Deref for Uri {
	type Target = UriRef;

	fn deref(&self) -> &Self::Target {
		self.as_uri_ref()
	}
}

// crate::common::owned!(Uri, UriBuf, UriRef, UriRefBuf);

impl UriBuf {
	pub fn into_uri_ref(self) -> UriRefBuf {
		unsafe { UriRefBuf::new_unchecked(self.0) }
	}

	pub fn into_iri(self) -> IriBuf {
		unsafe { IriBuf::new_unchecked(self.0) }
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(self.0) }
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
