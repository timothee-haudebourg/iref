use std::{
	hash::{Hash, Hasher},
	ops::Deref,
};

/// URI authority user info.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(super::super::grammar::UserInfo)]
#[newtype(
	no_deref,
	ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
	owned(
		UserInfoBuf,
		derive(PartialEq, Eq, PartialOrd, Ord, Hash)
	)
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct UserInfo(str);

impl UserInfo {
	/// Returns the host as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &pct_str::PctStr {
		unsafe { pct_str::PctStr::new_unchecked(self.as_str()) }
	}
}

impl Deref for UserInfo {
	type Target = pct_str::PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for UserInfo {
	#[inline]
	fn eq(&self, other: &UserInfo) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for UserInfo {}

impl PartialOrd for UserInfo {
	#[inline]
	fn partial_cmp(&self, other: &UserInfo) -> Option<::core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for UserInfo {
	#[inline]
	fn cmp(&self, other: &UserInfo) -> ::core::cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for UserInfo {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl UserInfoBuf {
	pub fn into_pct_string(self) -> pct_str::PctString {
		unsafe { pct_str::PctString::new_unchecked(self.0) }
	}
}

/// Parses a URI authority [`UserInfo`] at compile time.
#[macro_export]
macro_rules! user_info {
	($value:literal) => {
		match $crate::uri::UserInfo::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI authority user info"),
		}
	};
}
