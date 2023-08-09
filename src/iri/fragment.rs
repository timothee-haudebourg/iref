use pct_str::{PctStr, PctString};
use std::{
	cmp, fmt,
	hash::{self, Hash},
	ops,
};

use static_regular_grammar::RegularGrammar;

/// IRI fragment.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "ifragment",
	no_deref,
	cache = "automata/iri/fragment.aut.cbor"
)]
#[grammar(sized(
	FragmentBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Fragment(str);

impl Fragment {
	/// Returns the fragment as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}
}

impl ops::Deref for Fragment {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl fmt::Display for Fragment {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for Fragment {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl cmp::PartialEq for Fragment {
	#[inline]
	fn eq(&self, other: &Fragment) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Fragment {}

impl<'a> PartialEq<&'a str> for Fragment {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialOrd for Fragment {
	#[inline]
	fn partial_cmp(&self, other: &Fragment) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Fragment {
	#[inline]
	fn cmp(&self, other: &Fragment) -> cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Fragment {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl FragmentBuf {
	pub fn into_pct_string(self) -> PctString {
		unsafe { PctString::new_unchecked(self.0) }
	}
}
