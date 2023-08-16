use crate::uri::{Scheme, SchemeBuf};

pub mod authority;
pub mod authority_mut;
pub mod fragment;
pub mod parse;
pub mod path;
pub mod path_mut;
pub mod query;
pub mod reference;

pub use authority::*;
pub use authority_mut::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use reference::*;

pub trait RiImpl: RiRefImpl {
	/// Returns the scheme of the IRI.
	#[inline]
	fn scheme(&self) -> &Scheme {
		let bytes = self.as_bytes();
		let range = parse::scheme(bytes, 0);
		unsafe { Scheme::new_unchecked(&bytes[range]) }
	}
}

pub trait RiBufImpl: Sized + RiRefBufImpl {
	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self;

	#[inline]
	fn from_scheme(scheme: SchemeBuf) -> Self {
		let mut bytes = scheme.into_bytes();
		bytes.push(b':');
		unsafe { Self::new_unchecked(bytes) }
	}

	#[inline]
	fn set_scheme(&mut self, new_scheme: &Scheme) {
		let range = parse::scheme(self.as_bytes(), 0);
		unsafe { self.replace(range, new_scheme.as_bytes()) }
	}
}
