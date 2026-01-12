use std::{
	cmp::Ordering,
	hash::{Hash, Hasher},
	ops::Deref,
};

use pct_str::{PctStr, PctString};

/// URI authority host.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(crate::uri::grammar::Host)]
#[newtype(
	no_deref,
	ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
	owned(HostBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Host(str);

impl Host {
	/// Returns the host as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}
}

impl Deref for Host {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for Host {
	#[inline]
	fn eq(&self, other: &Host) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Host {}

impl PartialOrd for Host {
	#[inline]
	fn partial_cmp(&self, other: &Host) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Host {
	#[inline]
	fn cmp(&self, other: &Host) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Host {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl HostBuf {
	pub fn into_pct_string(self) -> PctString {
		unsafe { PctString::new_unchecked(self.0) }
	}
}

/// Parses a URI authority [`Host`] at compile time.
#[macro_export]
macro_rules! host {
	($value:literal) => {
		match $crate::uri::Host::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI authority host"),
		}
	};
}
