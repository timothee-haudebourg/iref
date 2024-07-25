use static_regular_grammar::RegularGrammar;

#[derive(RegularGrammar, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[grammar(
	file = "crates/core/src/uri/grammar.abnf",
	entry_point = "port",
	name = "URI port",
	ascii,
	cache = "crates/core/automata/uri/port.aut.cbor"
)]
#[grammar(sized(PortBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Port([u8]);
