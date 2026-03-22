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

macro_rules! port_as_int {
	($(($method:ident, $ty:ident)),*) => {
		impl Port {
			$(
				/// Tries to parse the port as a
				#[doc = "`"]
				#[doc = stringify!($ty)]
				#[doc = "`"]
				/// numeric value.
				///
				/// Returns `None` if the port number is out of range.
				pub fn $method(&self) -> Option<$ty> {
					self.as_str().parse().ok()
				}
			)*
		}

		$(
			impl TryFrom<&Port> for $ty {
				type Error = core::num::ParseIntError;

				fn try_from(port: &Port) -> Result<Self, Self::Error> {
					port.as_str().parse()
				}
			}
		)*
	};
}

port_as_int!(
	(as_u8, u8),
	(as_u16, u16),
	(as_u32, u32),
	(as_u64, u64),
	(as_u128, u128),
	(as_i8, i8),
	(as_i16, i16),
	(as_i32, i32),
	(as_i64, i64),
	(as_i128, i128)
);

impl Port {
	pub const HTTP: &Self = port!("80");
	pub const HTTPS: &Self = port!("443");
}

#[cfg(feature = "std")]
mod port_buf {
	macro_rules! port_from_uint {
		($($ty:ident),*) => {
			$(
				impl From<$ty> for super::PortBuf {
					fn from(value: $ty) -> Self {
						unsafe { Self::new_unchecked(value.to_string()) }
					}
				}
			)*
		};
	}

	port_from_uint!(u8, u16, u32, u64, u128);
}
