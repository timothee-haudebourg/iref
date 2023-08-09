use pct_str::PctStr;
use std::{
	cmp, fmt,
	hash::{self, Hash},
	ops,
};

use static_regular_grammar::RegularGrammar;

use crate::iri::Scheme;

/// IRI path segment.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "isegment",
	no_deref,
	cache = "automata/iri/segment.aut.cbor"
)]
#[grammar(sized(
	SegmentBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Segment(str);

impl Segment {
	pub const CURRENT: &'static Self = unsafe { Segment::new_unchecked(".") };

	pub const PARENT: &'static Self = unsafe { Segment::new_unchecked("..") };

	/// Returns the segment as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	/// Checks if this segments looks like a scheme part.
	///
	/// Returns `true` is of the form `prefix:suffix` where `prefix` is a valid
	/// scheme, of `false` otherwise.
	#[inline]
	pub fn looks_like_scheme(&self) -> bool {
		self.0
			.split_once(':')
			.is_some_and(|(prefix, _)| Scheme::new(prefix.as_bytes()).is_ok())
	}
}

impl ops::Deref for Segment {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl fmt::Display for Segment {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for Segment {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
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
