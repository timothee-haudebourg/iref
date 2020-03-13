use std::ops::Deref;
use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use pct_str::PctStr;
use crate::IriRefBuf;
use super::{Iri, Error, Scheme, Authority, AuthorityMut, Path, PathMut, Query, Fragment};

/// Owned IRI.
pub struct IriBuf(IriRefBuf);

impl IriBuf {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<IriBuf, Error> {
		let iri_ref = IriRefBuf::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(Error::Invalid)
		}
	}

	pub fn as_iri(&self) -> Iri {
		Iri(self.0.as_iri_ref())
	}

	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}

	/// Set the scheme of the IRI.
	pub fn set_scheme(&mut self, scheme: Scheme) {
		self.0.set_scheme(Some(scheme))
	}

	pub fn authority_mut(&mut self) -> AuthorityMut {
		self.0.authority_mut()
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_authority(&mut self, authority: Authority) {
		self.0.set_authority(authority)
	}

	pub fn path_mut<'a>(&'a mut self) -> PathMut<'a> {
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

impl<'a> cmp::PartialEq for Iri<'a> {
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri_ref().eq(other.as_iri_ref())
	}
}

impl<'a> Eq for Iri<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Iri<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_iri_ref().eq(other)
	}
}

impl<'a> Hash for Iri<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}

impl<'a> From<Iri<'a>> for IriBuf {
	fn from(iri: Iri<'a>) -> IriBuf {
		IriBuf::new(iri.as_str()).unwrap()
	}
}
