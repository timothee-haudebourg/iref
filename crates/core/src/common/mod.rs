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

macro_rules! str_eq {
	($ident:ident) => {
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

pub(crate) use str_eq;

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
