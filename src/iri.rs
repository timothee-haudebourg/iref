use static_regular_grammar::RegularGrammar;

mod authority;
mod authority_mut;
mod fragment;
mod path;
mod path_mut;
mod query;
mod reference;

pub use authority::*;
pub use authority_mut::*;
pub use fragment::*;
pub use path::*;
pub use path_mut::*;
pub use query::*;
pub use reference::*;

use crate::common::{parse, RiBufImpl, RiImpl, RiRefBufImpl, RiRefImpl};

/// IRI.
///
/// Wrapper around a borrowed bytes slice representing an IRI.
/// An IRI can be seen as an IRI-reference with a defined [`Scheme`].
/// All methods of [`IriRef`] are available from this type, however the [`scheme`](Iri::scheme) method
/// is redefined to always return some scheme.
///
/// # Example
///
/// ```rust
/// use iref::iri::{Iri, Scheme, Authority, Path, Query, Fragment};
/// # fn main() -> Result<(), iref::iri::InvalidIri<&'static str>> {
/// let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#fragment")?;
///
/// assert_eq!(iri.scheme(), Scheme::new(b"https").unwrap());
/// assert_eq!(iri.authority(), Some(Authority::new("www.rust-lang.org").unwrap()));
/// assert_eq!(iri.path(), Path::new("/foo/bar").unwrap());
/// assert_eq!(iri.query(), Some(Query::new("query").unwrap()));
/// assert_eq!(iri.fragment(), Some(Fragment::new("fragment").unwrap()));
/// #
/// # Ok(())
/// # }
/// ```
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI",
	cache = "automata/iri.aut.cbor"
)]
#[grammar(sized(IriBuf))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Iri(str);

pub struct IriParts<'a> {
	pub scheme: &'a Scheme,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

impl RiRefImpl for Iri {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiImpl for Iri {}

impl Iri {
	pub fn parts(&self) -> IriParts {
		let bytes = self.as_bytes();
		let ranges = parse::parts(bytes, 0);

		IriParts {
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

	/// Returns the scheme of the IRI.
	#[inline]
	pub fn scheme(&self) -> &Scheme {
		RiImpl::scheme(self)
	}

	/// Returns the authority part of the IRI reference, if any.
	pub fn authority(&self) -> Option<&Authority> {
		RiRefImpl::authority(self)
	}

	/// Returns the path of the IRI reference.
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

impl RiRefImpl for IriBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiImpl for IriBuf {}

impl RiRefBufImpl for IriBuf {
	type Ri = Iri;
	type RiBuf = Self;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	fn into_bytes(self) -> Vec<u8> {
		self.0.into_bytes()
	}
}

impl RiBufImpl for IriBuf {
	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
		Self::new_unchecked(String::from_utf8_unchecked(bytes))
	}
}

impl IriBuf {
	pub fn from_scheme(scheme: SchemeBuf) -> Self {
		RiBufImpl::from_scheme(scheme)
	}
}
