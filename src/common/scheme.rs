use core::{
	cmp::Ordering,
	hash::{Hash, Hasher},
};
use static_automata::Validate;
use str_newtype::StrNewType;

/// URI or IRI scheme.
#[derive(Validate, StrNewType)]
#[automaton(super::grammar::Scheme)]
#[newtype(
	ord([u8], &[u8], str, &str),
)]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String), owned(SchemeBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Scheme(str);

/// Parses an IRI/IRI [`Scheme`] at compile time.
#[macro_export]
macro_rules! scheme {
	($value:literal) => {
		match $crate::iri::Scheme::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI/IRI scheme"),
		}
	};
}

impl Scheme {
	pub const HTTP: &Self = scheme!("http");
	pub const HTTPS: &Self = scheme!("https");
	pub const FILE: &Self = scheme!("file");
	pub const FTP: &Self = scheme!("ftp");
	pub const URN: &Self = scheme!("urn");
	pub const DATA: &Self = scheme!("data");
	pub const MAILTO: &Self = scheme!("mailto");
}

impl PartialEq for Scheme {
	fn eq(&self, other: &Self) -> bool {
		self.as_str().eq_ignore_ascii_case(other.as_str())
	}
}

impl Eq for Scheme {}

impl PartialOrd for Scheme {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Scheme {
	fn cmp(&self, other: &Self) -> Ordering {
		self.chars()
			.map(|c| c.to_ascii_lowercase())
			.cmp(other.chars().map(|c| c.to_ascii_lowercase()))
	}
}

impl Hash for Scheme {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.chars().map(|c| c.to_ascii_lowercase()) {
			state.write_u32(c as u32);
		}
	}
}
