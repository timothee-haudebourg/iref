use super::Error;
use crate::parsing;
use pct_str::PctStr;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

#[derive(Clone, Copy)]
pub struct UserInfo<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8],
}

impl<'a> UserInfo<'a> {
	#[inline]
	pub fn as_ref(&self) -> &[u8] {
		self.data
	}

	/// Get the underlying userinfo slice as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.data) }
	}

	/// Get the underlying userinfo slice as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	/// Checks if the userinfo is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for UserInfo<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<UserInfo<'a>, Error> {
		let userinfo_len = parsing::parse_userinfo(str.as_ref(), 0)?;
		if userinfo_len < str.len() {
			Err(Error::InvalidUserInfo)
		} else {
			Ok(UserInfo { data: str.as_ref() })
		}
	}
}

impl<'a> fmt::Display for UserInfo<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for UserInfo<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for UserInfo<'a> {
	#[inline]
	fn eq(&self, other: &UserInfo) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for UserInfo<'a> {}

impl<'a> cmp::PartialEq<&'a str> for UserInfo<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> PartialOrd for UserInfo<'a> {
	#[inline]
	fn partial_cmp(&self, other: &UserInfo<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for UserInfo<'a> {
	#[inline]
	fn cmp(&self, other: &UserInfo<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl<'a> Hash for UserInfo<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
