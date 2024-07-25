use static_regular_grammar::RegularGrammar;

#[cfg(feature = "data")]
pub mod data;

#[derive(RegularGrammar, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[grammar(
	file = "crates/core/src/uri/grammar.abnf",
	entry_point = "scheme",
	name = "Scheme",
	ascii,
	cache = "crates/core/automata/uri/scheme.aut.cbor"
)]
#[grammar(sized(
	SchemeBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Scheme([u8]);
