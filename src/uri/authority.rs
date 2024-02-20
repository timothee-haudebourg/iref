use std::{
	cmp,
	hash::{self, Hash},
};

use static_regular_grammar::RegularGrammar;

mod host;
mod port;
mod userinfo;

use crate::common::AuthorityImpl;

pub use host::*;
pub use port::*;
pub use userinfo::*;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "authority",
	name = "URI authority",
	ascii,
	cache = "automata/uri/authority.aut.cbor"
)]
#[grammar(sized(
	AuthorityBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Authority([u8]);

impl AuthorityImpl for Authority {
	type UserInfo = UserInfo;
	type Host = Host;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(bytes)
	}

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityParts<'a> {
	pub user_info: Option<&'a UserInfo>,
	pub host: &'a Host,
	pub port: Option<&'a Port>,
}

impl Authority {
	pub fn user_info(&self) -> Option<&UserInfo> {
		AuthorityImpl::user_info(self)
	}

	pub fn host(&self) -> &Host {
		AuthorityImpl::host(self)
	}

	pub fn port(&self) -> Option<&Port> {
		AuthorityImpl::port(self)
	}

	pub fn parts(&self) -> AuthorityParts {
		let ranges = AuthorityImpl::parts(self);

		AuthorityParts {
			user_info: ranges
				.user_info
				.map(|r| unsafe { UserInfo::new_unchecked(&self.0[r]) }),
			host: unsafe { Host::new_unchecked(&self.0[ranges.host]) },
			port: ranges
				.port
				.map(|r| unsafe { Port::new_unchecked(&self.0[r]) }),
		}
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
