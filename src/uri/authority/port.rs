use static_automata::Validate;
use str_newtype::StrNewType;

/// URI/IRI authority port.
#[derive(Validate, StrNewType, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[automaton(crate::uri::grammar::Port)]
#[newtype(ord([u8], &[u8], Vec<u8>, str, &str, String), owned(PortBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Port(str);

/// Parses a URI/IRI authority [`Port`] at compile time.
#[macro_export]
macro_rules! port {
	($value:literal) => {
		match $crate::uri::Port::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI/IRI authority port"),
		}
	};
}
