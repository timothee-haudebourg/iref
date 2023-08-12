use std::{
	cmp, fmt,
	hash::{self, Hash},
};

use static_regular_grammar::RegularGrammar;

mod host;
mod port;
mod userinfo;

use crate::common::AuthorityImpl;

pub use super::{InvalidScheme, Scheme, SchemeBuf};
pub use host::*;
pub use port::*;
pub use userinfo::*;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "authority",
	ascii,
	cache = "automata/uri/authority.aut.cbor"
)]
#[grammar(sized(
	AuthorityBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
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

	pub fn parts(&self) -> (Option<&UserInfo>, &Host, Option<&Port>) {
		todo!()
	}
}

impl cmp::PartialEq for Authority {
	#[inline]
	fn eq(&self, other: &Authority) -> bool {
		let (u1, h1, p1) = self.parts();
		let (u2, h2, p2) = other.parts();
		u1 == u2 && h1 == h2 && p1 == p2
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
		let (u1, h1, p1) = self.parts();
		let (u2, h2, p2) = other.parts();
		u1.cmp(&u2)
			.then_with(|| h1.cmp(h2))
			.then_with(|| p1.cmp(&p2))
	}
}

impl Hash for Authority {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		let (u, h, p) = self.parts();
		u.hash(hasher);
		h.hash(hasher);
		p.hash(hasher)
	}
}
