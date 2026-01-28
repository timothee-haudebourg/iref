use static_automata::Validate;
use str_newtype::StrNewType;

/// URI/IRI authority port.
#[derive(Validate, StrNewType, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[automaton(super::grammar::Port)]
#[newtype(ord([u8], &[u8], str, &str))]
#[cfg_attr(feature = "std", newtype(
    ord(Vec<u8>, String), owned(PortBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash))
))]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Port(str);

/// Parses a URI/IRI authority [`Port`] at compile time.
#[macro_export]
macro_rules! port {
	($value:literal) => {
		match $crate::Port::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI/IRI authority port"),
		}
	};
}

impl Port {
	pub const HTTP: &Self = port!("80");
	pub const HTTPS: &Self = port!("443");
}

#[cfg(feature = "std")]
mod port_buf {
	macro_rules! port_from_uint {
		($ty:ident) => {
			impl From<$ty> for super::PortBuf {
				fn from(value: $ty) -> Self {
					unsafe { Self::new_unchecked(value.to_string()) }
				}
			}
		};
	}

	port_from_uint!(u8);
	port_from_uint!(u16);
	port_from_uint!(u32);
	port_from_uint!(u64);
}
