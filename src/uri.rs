use static_regular_grammar::RegularGrammar;

mod authority;
mod fragment;
mod query;
mod scheme;

pub use authority::*;
pub use fragment::*;
pub use query::*;
pub use scheme::*;

use crate::parse;

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

pub struct UriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	// pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl Uri {
	pub fn parts(&self) -> UriParts {
		let bytes = self.as_bytes();
		let ranges = parse::parts(bytes, 0);

		UriParts {
			scheme: unsafe { Scheme::new_unchecked(&bytes[ranges.scheme]) },
			authority: ranges.authority
				.map(|r| unsafe { Authority::new_unchecked(&self.0[r]) }),
			// path: unsafe { Path::new_unchecked(&self.0[ranges.path]) },
			query: ranges.query
				.map(|r| unsafe { Query::new_unchecked(&self.0[r]) }),
			fragment: ranges.fragment
				.map(|r| unsafe { Fragment::new_unchecked(&self.0[r]) }),
		}
	}

	/// Returns the scheme of the IRI.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		let bytes = self.as_bytes();
		let range = parse::scheme(bytes, 0);
		unsafe {
			Scheme::new_unchecked(&bytes[range])
		}
	}

	/// Returns the authority part of the IRI reference, if any.
	pub fn authority(&self) -> Option<&Authority> {
		parse::find_authority(self.as_bytes(), 0).map(|range| unsafe {
			Authority::new_unchecked(&self.0[range])
		})
	}

	// /// Returns the path of the IRI reference.
	// pub fn path(&self) -> &Path {
	// 	let range = parse::find_path(self.as_bytes(), 0);
	// 	unsafe {
	// 		Path::new_unchecked(&self.0[range])
	// 	}
	// }

	pub fn query(&self) -> Option<&Query> {
		parse::find_query(self.as_bytes(), 0).map(|range| {
			unsafe { Query::new_unchecked(&self.0[range]) }
		})
	}

	pub fn fragment(&self) -> Option<&Fragment> {
		parse::find_fragment(self.as_bytes(), 0).map(|range| {
			unsafe { Fragment::new_unchecked(&self.0[range]) }
		})
	}
}
