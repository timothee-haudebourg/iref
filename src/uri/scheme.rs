use static_regular_grammar::RegularGrammar;

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
