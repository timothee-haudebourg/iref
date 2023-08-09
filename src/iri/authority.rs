use std::{
	cmp, fmt,
	hash::{self, Hash},
};

use static_regular_grammar::RegularGrammar;

mod host;
mod userinfo;

pub use crate::uri::{InvalidPort, Port, PortBuf};
pub use crate::uri::{InvalidScheme, Scheme, SchemeBuf};
pub use host::*;
pub use userinfo::*;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "iauthority",
	cache = "automata/iri/authority.aut.cbor"
)]
#[grammar(sized(
	AuthorityBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Authority(str);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityParts<'a> {
	pub user_info: Option<&'a UserInfo>,
	pub host: &'a Host,
	pub port: Option<&'a Port>,
}

impl Authority {
	pub fn user_info(&self) -> Option<&UserInfo> {
		todo!()
	}

	pub fn host(&self) -> &Host {
		todo!()
	}

	pub fn port(&self) -> Option<&Port> {
		todo!()
	}

	pub fn parts(&self) -> AuthorityParts {
		let mut user_info_end = None;
		let mut host_start = 0;
		let mut host_end = None;
		let mut port_start = None;

		for (i, c) in self.0.char_indices() {
			match c {
				'@' if user_info_end.is_none() => {
					if port_start.is_some() {
						port_start = None;
						host_end = None
					}

					user_info_end = Some(i);
					host_start = i + 1
				}
				':' if port_start.is_none() => {
					port_start = Some(i + 1);
					host_end = Some(i)
				}
				_ => (),
			}
		}

		AuthorityParts {
			user_info: user_info_end.map(|e| unsafe { UserInfo::new_unchecked(&self.0[..e]) }),
			host: unsafe {
				Host::new_unchecked(&self.0[host_start..host_end.unwrap_or(self.0.len())])
			},
			port: port_start.map(|s| unsafe { Port::new_unchecked(&self.0.as_bytes()[s..]) }),
		}
	}
}

impl fmt::Display for Authority {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for Authority {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl cmp::PartialEq for Authority {
	#[inline]
	fn eq(&self, other: &Authority) -> bool {
		self.parts() == other.parts()
	}
}

impl Eq for Authority {}

impl<'a> PartialEq<&'a str> for Authority {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialOrd for Authority {
	#[inline]
	fn partial_cmp(&self, other: &Authority) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Authority {
	#[inline]
	fn cmp(&self, other: &Authority) -> cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for Authority {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.parts().hash(hasher)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parts() {
		let vectors = [
			("host", (None, "host", None)),
			("user@host", (Some("user"), "host", None)),
			("host:123", (None, "host", Some("123"))),
			("user@host:123", (Some("user"), "host", Some("123"))),
			("a:b@host", (Some("a:b"), "host", None)),
			("a:b@host:123", (Some("a:b"), "host", Some("123"))),
		];

		for (input, expected) in vectors {
			eprintln!("{input} => {expected:?}");
			let input = Authority::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.user_info.map(UserInfo::as_str), expected.0);
			assert_eq!(parts.host.as_str(), expected.1);
			assert_eq!(parts.port.map(Port::as_str), expected.2)
		}
	}
}
