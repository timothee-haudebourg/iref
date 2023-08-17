use pct_str::PctStr;
use std::{
	cmp,
	hash::{self, Hash},
	ops,
};

use static_regular_grammar::RegularGrammar;

use crate::common::path::SegmentImpl;

/// IRI path segment.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "segment",
	ascii,
	no_deref,
	cache = "automata/uri/segment.aut.cbor"
)]
#[grammar(sized(
	SegmentBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Segment([u8]);

impl SegmentImpl for Segment {
	const PARENT: &'static Self = Self::PARENT;

	const EMPTY: &'static Self = Self::EMPTY;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(bytes)
	}

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl Segment {
	pub const CURRENT: &'static Self = unsafe { Segment::new_unchecked(b".") };

	pub const PARENT: &'static Self = unsafe { Segment::new_unchecked(b"..") };

	/// Returns the segment as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		SegmentImpl::as_pct_str(self)
	}

	/// Checks if this segments looks like a scheme part.
	///
	/// Returns `true` is of the form `prefix:suffix` where `prefix` is a valid
	/// scheme, of `false` otherwise.
	#[inline]
	pub fn looks_like_scheme(&self) -> bool {
		SegmentImpl::looks_like_scheme(self)
	}
}

impl ops::Deref for Segment {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for Segment {
	fn eq(&self, other: &Self) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Segment {}

impl PartialOrd for Segment {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.as_pct_str().partial_cmp(other.as_pct_str())
	}
}

impl Ord for Segment {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Segment {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.as_pct_str().hash(state)
	}
}
