use static_regular_grammar::RegularGrammar;

mod authority;
mod fragment;
mod path;
mod path_mut;
mod query;
mod scheme;

pub use authority::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use scheme::*;

use crate::common::{parse, RiImpl, RiRefImpl};

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "URI",
	ascii,
	cache = "automata/uri.aut.cbor"
)]
#[grammar(sized(UriBuf))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Uri([u8]);

impl RiRefImpl for Uri {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl RiImpl for Uri {}

pub struct UriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl Uri {
	pub fn parts(&self) -> UriParts {
		let bytes = self.as_bytes();
		let ranges = parse::parts(bytes, 0);

		UriParts {
			scheme: unsafe { Scheme::new_unchecked(&bytes[ranges.scheme]) },
			authority: ranges
				.authority
				.map(|r| unsafe { Authority::new_unchecked(&self.0[r]) }),
			path: unsafe { Path::new_unchecked(&self.0[ranges.path]) },
			query: ranges
				.query
				.map(|r| unsafe { Query::new_unchecked(&self.0[r]) }),
			fragment: ranges
				.fragment
				.map(|r| unsafe { Fragment::new_unchecked(&self.0[r]) }),
		}
	}

	/// Returns the scheme of the URI.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		RiImpl::scheme(self)
	}

	/// Returns the authority part of the URI, if any.
	pub fn authority(&self) -> Option<&Authority> {
		RiRefImpl::authority(self)
	}

	/// Returns the path of the URI.
	pub fn path(&self) -> &Path {
		RiRefImpl::path(self)
	}

	pub fn query(&self) -> Option<&Query> {
		RiRefImpl::query(self)
	}

	pub fn fragment(&self) -> Option<&Fragment> {
		RiRefImpl::fragment(self)
	}
}
