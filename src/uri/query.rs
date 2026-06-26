/// URI query.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(super::grammar::Query)]
#[newtype(
	no_deref,
	ord([u8], &[u8], str, &str, pct_str::PctStr, &pct_str::PctStr)
)]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String, pct_str::PctString), owned(QueryBuf, derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Query(str);

impl Default for &Query {
	fn default() -> Self {
		Query::EMPTY
	}
}

impl Query {
	/// The empty query.
	pub const EMPTY: &'static Self = match Self::from_str("") {
		Ok(v) => v,
		Err(_) => panic!("empty query should be valid"),
	};

	/// Returns the query as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &pct_str::PctStr {
		unsafe { pct_str::PctStr::new_unchecked(self.as_str()) }
	}
}

impl core::ops::Deref for Query {
	type Target = pct_str::PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for Query {
	#[inline]
	fn eq(&self, other: &Query) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Query {}

impl PartialOrd for Query {
	#[inline]
	fn partial_cmp(&self, other: &Query) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Query {
	#[inline]
	fn cmp(&self, other: &Query) -> core::cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl core::hash::Hash for Query {
	#[inline]
	fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

#[cfg(feature = "std")]
impl QueryBuf {
	pub fn into_pct_string(self) -> pct_str::PctString {
		unsafe { pct_str::PctString::new_unchecked(self.0) }
	}
}

/// Parses an URI [`Query`] at compile time.
#[macro_export]
macro_rules! query {
	($value:literal) => {
		match $crate::uri::Query::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI query"),
		}
	};
}
