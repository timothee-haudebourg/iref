use static_regular_grammar::RegularGrammar;

use crate::{
	common::{parse, RiRefBufImpl, RiRefImpl},
	Iri, IriBuf, RiRefParts,
};

use super::{Authority, Fragment, Path, Query, Scheme};

/// IRI reference.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI-reference",
	cache = "automata/iri/reference.aut.cbor"
)]
#[grammar(sized(IriRefBuf, derive(Debug, Display)))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct IriRef(str);

impl RiRefImpl for IriRef {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

pub type IriRefParts<'a> = RiRefParts<'a, IriRef>;

impl IriRef {
	pub fn parts(&self) -> IriRefParts {
		let bytes = self.as_bytes();
		let ranges = parse::reference_parts(bytes, 0);

		IriRefParts {
			scheme: ranges
				.scheme
				.map(|r| unsafe { Scheme::new_unchecked(&bytes[r]) }),
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

	/// Returns the scheme of the IRI reference, if any.
	#[inline]
	pub fn scheme(&self) -> Option<&Scheme> {
		RiRefImpl::scheme_opt(self)
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

	/// Resolve the IRI reference against the given *base IRI*.
	///
	/// Return the resolved IRI.
	/// See the [`IriRefBuf::resolve`] method for more information about the resolution process.
	#[inline]
	pub fn resolved(&self, base_iri: &impl AsRef<Iri>) -> IriBuf {
		let iri_ref = self.to_owned();
		iri_ref.resolved(base_iri)
	}
}

impl RiRefImpl for IriRefBuf {
	type Authority = Authority;
	type Path = Path;
	type Query = Query;
	type Fragment = Fragment;

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl RiRefBufImpl for IriRefBuf {
	type Ri = Iri;
	type RiBuf = IriBuf;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	fn into_bytes(self) -> Vec<u8> {
		self.0.into_bytes()
	}
}

impl IriRefBuf {
	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	/// Resolve the IRI reference.
	///
	/// ## Abnormal use of dot segments.
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn resolve(&mut self, base_iri: &impl AsRef<Iri>) {
		RiRefBufImpl::resolve(self, base_iri.as_ref())
	}

	pub fn resolved(self, base_iri: &impl AsRef<Iri>) -> IriBuf {
		RiRefBufImpl::resolved(self, base_iri.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PARTS: [(
		&str,
		(Option<&str>, Option<&str>, &str, Option<&str>, Option<&str>),
	); 36] = [
		// 0 components.
		("", (None, None, "", None, None)),
		// 1 component.
		("scheme:", (Some("scheme"), None, "", None, None)),
		("//authority", (None, Some("authority"), "", None, None)),
		("path", (None, None, "path", None, None)),
		("/path", (None, None, "/path", None, None)),
		("/", (None, None, "/", None, None)),
		("foo//bar", (None, None, "foo//bar", None, None)),
		("?query", (None, None, "", Some("query"), None)),
		("#fragment", (None, None, "", None, Some("fragment"))),
		(
			"scheme:?query",
			(Some("scheme"), None, "", Some("query"), None),
		),
		// 2 components.
		(
			"scheme://authority",
			(Some("scheme"), Some("authority"), "", None, None),
		),
		("scheme:path", (Some("scheme"), None, "path", None, None)),
		("scheme:/path", (Some("scheme"), None, "/path", None, None)),
		(
			"scheme:?query",
			(Some("scheme"), None, "", Some("query"), None),
		),
		(
			"scheme:#fragment",
			(Some("scheme"), None, "", None, Some("fragment")),
		),
		(
			"//authority/path",
			(None, Some("authority"), "/path", None, None),
		),
		(
			"//authority?query",
			(None, Some("authority"), "", Some("query"), None),
		),
		(
			"//authority#fragment",
			(None, Some("authority"), "", None, Some("fragment")),
		),
		("path?query", (None, None, "path", Some("query"), None)),
		("/path?query", (None, None, "/path", Some("query"), None)),
		(
			"path#fragment",
			(None, None, "path", None, Some("fragment")),
		),
		(
			"?query#fragment",
			(None, None, "", Some("query"), Some("fragment")),
		),
		// 3 components
		(
			"scheme://authority/path",
			(Some("scheme"), Some("authority"), "/path", None, None),
		),
		(
			"scheme://authority?query",
			(Some("scheme"), Some("authority"), "", Some("query"), None),
		),
		(
			"scheme://authority#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"",
				None,
				Some("fragment"),
			),
		),
		(
			"scheme:path?query",
			(Some("scheme"), None, "path", Some("query"), None),
		),
		(
			"scheme:path#fragment",
			(Some("scheme"), None, "path", None, Some("fragment")),
		),
		(
			"//authority/path?query",
			(None, Some("authority"), "/path", Some("query"), None),
		),
		(
			"//authority/path#fragment",
			(None, Some("authority"), "/path", None, Some("fragment")),
		),
		(
			"//authority?query#fragment",
			(None, Some("authority"), "", Some("query"), Some("fragment")),
		),
		(
			"path?query#fragment",
			(None, None, "path", Some("query"), Some("fragment")),
		),
		// 4 components
		(
			"scheme://authority/path?query",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				Some("query"),
				None,
			),
		),
		(
			"scheme://authority/path#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				None,
				Some("fragment"),
			),
		),
		(
			"scheme://authority?query#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"",
				Some("query"),
				Some("fragment"),
			),
		),
		(
			"scheme:path?query#fragment",
			(
				Some("scheme"),
				None,
				"path",
				Some("query"),
				Some("fragment"),
			),
		),
		// 5 components
		(
			"scheme://authority/path?query#fragment",
			(
				Some("scheme"),
				Some("authority"),
				"/path",
				Some("query"),
				Some("fragment"),
			),
		),
	];

	#[test]
	fn parts() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.scheme.map(Scheme::as_str), expected.0);
			assert_eq!(parts.authority.map(Authority::as_str), expected.1);
			assert_eq!(parts.path.as_str(), expected.2);
			assert_eq!(parts.query.map(Query::as_str), expected.3);
			assert_eq!(parts.fragment.map(Fragment::as_str), expected.4)
		}
	}

	#[test]
	fn scheme() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			assert_eq!(input.scheme().map(Scheme::as_str), expected.0)
		}
	}

	#[test]
	fn authority() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			eprintln!("{input}: {expected:?}");
			assert_eq!(input.authority().map(Authority::as_str), expected.1)
		}
	}

	#[test]
	fn set_authority() {
		let vectors = [
			("scheme:path", Some("authority"), "scheme://authority/path"),
			("scheme://authority//path", None, "scheme:/.//path"),
		];

		for (input, authority, expected) in vectors {
			let mut buffer = IriRefBuf::new(input.to_string()).unwrap();
			let authority = authority.map(Authority::new).transpose().unwrap();
			buffer.set_authority(authority);
			eprintln!("{input}, {authority:?} => {buffer}, {expected}");
			assert_eq!(buffer.as_str(), expected)
		}
	}

	#[test]
	fn path() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			eprintln!("{input}: {expected:?}");
			assert_eq!(input.path().as_str(), expected.2)
		}
	}

	#[test]
	fn query() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			eprintln!("{input}: {expected:?}");
			assert_eq!(input.query().map(Query::as_str), expected.3)
		}
	}

	#[test]
	fn fragment() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			eprintln!("{input}: {expected:?}");
			assert_eq!(input.fragment().map(Fragment::as_str), expected.4)
		}
	}
}
