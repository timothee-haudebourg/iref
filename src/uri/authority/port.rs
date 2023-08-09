use static_regular_grammar::RegularGrammar;
use std::fmt;

#[derive(RegularGrammar, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "port",
	ascii,
	cache = "automata/uri/port.aut.cbor"
)]
#[grammar(sized(PortBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Port([u8]);

impl fmt::Debug for Port {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Display for Port {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
