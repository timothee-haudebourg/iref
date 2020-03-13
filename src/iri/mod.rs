mod scheme;
mod authority;
mod path;
mod buffer;
mod query;
mod fragment;

use std::ops::Deref;
use pct_str::PctStr;
use crate::parsing;
use crate::IriRef;

pub use self::scheme::*;
pub use self::authority::*;
pub use self::path::*;
pub use self::buffer::*;
pub use self::query::*;
pub use self::fragment::*;

pub type Error = crate::parsing::Error;

/// IRI slice.
///
/// Note that in future versions, this will most likely become a custom dynamic sized type,
/// similar to `str`.
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
	pub fn as_iri_ref(&self) -> &IriRef<'a> {
		&self.0
	}

	pub fn scheme(&self) -> &PctStr {
		self.0.scheme().unwrap()
	}
}

impl<'a> Deref for Iri<'a> {
	type Target = IriRef<'a>;

	fn deref(&self) -> &IriRef<'a> {
		self.as_iri_ref()
	}
}

impl<'a> From<&'a IriBuf> for Iri<'a> {
	fn from(buffer: &'a IriBuf) -> Iri<'a> {
		buffer.as_iri()
	}
}
