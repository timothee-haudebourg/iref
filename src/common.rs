use crate::uri::{Scheme, SchemeBuf};

pub mod parse;
pub mod authority;
pub mod authority_mut;
pub mod path;
pub mod path_mut;
pub mod query;
pub mod fragment;
pub mod reference;

pub use authority::*;
pub use authority_mut::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use fragment::*;
pub use reference::*;

pub struct RiParts<'a, T: RiImpl> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a T::Authority>,
	pub path: &'a T::Path,
	pub query: Option<&'a T::Query>,
	pub fragment: Option<&'a T::Fragment>,
}

pub trait RiImpl: RiRefImpl {
	/// Returns the scheme of the IRI.
	#[inline]
	fn scheme(&self) -> &Scheme {
		let bytes = self.as_bytes();
		let range = parse::scheme(bytes, 0);
		unsafe {
			Scheme::new_unchecked(&bytes[range])
		}
	}
}

pub trait RiBufImpl: Sized + RiRefBufImpl {
	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self;
	
	fn from_scheme(scheme: SchemeBuf) -> Self {
		let mut bytes = scheme.into_bytes();
		bytes.push(b':');
		unsafe {
			Self::new_unchecked(bytes)
		}
	}
}