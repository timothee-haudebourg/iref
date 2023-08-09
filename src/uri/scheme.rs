use static_regular_grammar::RegularGrammar;
use std::fmt;

#[derive(RegularGrammar, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "scheme",
	ascii,
	cache = "automata/uri/scheme.aut.cbor"
)]
#[grammar(sized(
	SchemeBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Scheme([u8]);

impl fmt::Debug for Scheme {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Display for Scheme {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
