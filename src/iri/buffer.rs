use std::ops::Deref;
use std::fmt;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use crate::IriRefBuf;
use super::{Iri, IriRef, Error, Scheme, Authority, AuthorityMut, Path, PathMut, Query, Fragment};

/// Owned IRI.
#[derive(Clone)]
pub struct IriBuf(pub(crate) IriRefBuf);

impl IriBuf {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<IriBuf, Error> {
		let iri_ref = IriRefBuf::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(Error::Invalid)
		}
	}

	pub fn from_scheme(scheme: Scheme) -> IriBuf {
		let mut iri_ref = IriRefBuf::default();
		iri_ref.set_scheme(Some(scheme));
		IriBuf(iri_ref)
	}

	pub fn as_iri(&self) -> Iri {
		Iri(self.0.as_iri_ref())
	}

	pub fn as_iri_ref(&self) -> IriRef {
		self.0.as_iri_ref()
	}

	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}

	/// Set the scheme of the IRI.
	pub fn set_scheme(&mut self, scheme: Scheme) {
		self.0.set_scheme(Some(scheme))
	}

	pub fn authority_mut(&mut self) -> Option<AuthorityMut> {
		self.0.authority_mut()
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_authority(&mut self, authority: Option<Authority>) {
		self.0.set_authority(authority)
	}

	pub fn path_mut(&mut self) -> PathMut {
		self.0.path_mut()
	}

	/// Set the IRI path.
	pub fn set_path(&mut self, path: Path) {
		self.0.set_path(path)
	}

	pub fn set_query(&mut self, query: Option<Query>) {
		self.0.set_query(query)
	}

	pub fn set_fragment(&mut self, fragment: Option<Fragment>) {
		self.0.set_fragment(fragment)
	}
}

impl Deref for IriBuf {
	type Target = IriRefBuf;

	fn deref(&self) -> &IriRefBuf {
		&self.0
	}
}

impl fmt::Display for IriBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl fmt::Debug for IriBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl PartialEq for IriBuf {
	fn eq(&self, other: &IriBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl Eq for IriBuf { }

impl<'a> PartialEq<Iri<'a>> for IriBuf {
	fn eq(&self, other: &Iri<'a>) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<IriRef<'a>> for IriBuf {
	fn eq(&self, other: &IriRef<'a>) -> bool {
		self.as_iri_ref() == *other
	}
}

impl PartialEq<IriRefBuf> for IriBuf {
	fn eq(&self, other: &IriRefBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a str> for IriBuf {
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Iri::new(other) {
			self == &other
		} else {
			false
		}
	}
}

impl PartialOrd for IriBuf {
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl Ord for IriBuf {
	fn cmp(&self, other: &IriBuf) -> Ordering {
		self.as_iri_ref().cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<Iri<'a>> for IriBuf {
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriRef<'a>> for IriBuf {
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl PartialOrd<IriRefBuf> for IriBuf {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<Iri<'a>> for IriBuf {
	fn from(iri: Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl<'a> From<&'a Iri<'a>> for IriBuf {
	fn from(iri: &'a Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl<'a> TryFrom<IriRef<'a>> for IriBuf {
	type Error = Error;

	fn try_from(iri_ref: IriRef<'a>) -> Result<IriBuf, Error> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(IriBuf(iri_ref.into()))
		} else {
			Err(Error::InvalidScheme)
		}
	}
}

impl TryFrom<IriRefBuf> for IriBuf {
	type Error = IriRefBuf;

	fn try_from(iri_ref: IriRefBuf) -> Result<IriBuf, IriRefBuf> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(iri_ref)
		}
	}
}

impl Hash for IriBuf {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}
