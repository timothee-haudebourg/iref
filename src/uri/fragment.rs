/// URI fragment.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(super::grammar::Fragment)]
#[newtype(
	no_deref,
	ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
	owned(FragmentBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Fragment(str);

impl Fragment {
	/// Returns the fragment as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &pct_str::PctStr {
		unsafe { pct_str::PctStr::new_unchecked(self.as_str()) }
	}
}

impl core::ops::Deref for Fragment {
	type Target = pct_str::PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for Fragment {
	#[inline]
	fn eq(&self, other: &Fragment) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Fragment {}

impl PartialOrd for Fragment {
	#[inline]
	fn partial_cmp(&self, other: &Fragment) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Fragment {
	#[inline]
	fn cmp(&self, other: &Fragment) -> core::cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl core::hash::Hash for Fragment {
	#[inline]
	fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl FragmentBuf {
	pub fn into_pct_string(self) -> pct_str::PctString {
		unsafe { pct_str::PctString::new_unchecked(self.0) }
	}
}

/// Parses an URI [`Fragment`] at compile time.
#[macro_export]
macro_rules! fragment {
	($value:literal) => {
		match $crate::uri::Fragment::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI fragment"),
		}
	};
}
