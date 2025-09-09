use static_automata::Validate;
use std::hash::Hash;
use str_newtype::StrNewType;

#[derive(Validate, StrNewType)]
#[automaton(super::grammar::Scheme)]
#[newtype(
	ord([u8], &[u8], Vec<u8>, str, &str, String),
	owned(
		SchemeBuf,
		derive(PartialEq, Eq, PartialOrd, Ord, Hash)
	)
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Scheme(str);

/// Parses an URI/IRI [`Scheme`] at compile time.
#[macro_export]
macro_rules! scheme {
	($value:literal) => {
		match $crate::uri::Scheme::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI/IRI scheme"),
		}
	};
}

impl PartialEq for Scheme {
	fn eq(&self, other: &Self) -> bool {
		self.as_str().eq_ignore_ascii_case(other.as_str())
	}
}

impl Eq for Scheme {}

impl PartialOrd for Scheme {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Scheme {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.chars()
			.map(|c| c.to_ascii_lowercase())
			.cmp(other.chars().map(|c| c.to_ascii_lowercase()))
	}
}

impl Hash for Scheme {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for c in self.chars().map(|c| c.to_ascii_lowercase()) {
			state.write_u32(c as u32);
		}
	}
}
