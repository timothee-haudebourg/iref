use std::hash::{self, Hash};

use static_regular_grammar::RegularGrammar;

use crate::{
	common::{parse, RiRefBufImpl, RiRefImpl},
	Iri, IriBuf,
};

use super::{Authority, AuthorityMut, Fragment, Path, PathMut, Query, Scheme};

/// IRI reference.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "IRI-reference",
	cache = "automata/iri/reference.aut.cbor"
)]
#[grammar(sized(
	IriRefBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IriRefParts<'a> {
	pub scheme: Option<&'a Scheme>,
	pub authority: Option<&'a Authority>,
	pub path: &'a Path,
	pub query: Option<&'a Query>,
	pub fragment: Option<&'a Fragment>,
}

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
	pub fn resolved(&self, base_iri: &(impl ?Sized + AsRef<Iri>)) -> IriBuf {
		let iri_ref = self.to_owned();
		iri_ref.into_resolved(base_iri)
	}
}

impl PartialEq<str> for IriRef {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl<'a> PartialEq<&'a str> for IriRef {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<String> for IriRef {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialEq for IriRef {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl Eq for IriRef {}

impl PartialOrd for IriRef {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for IriRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for IriRef {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
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

	/// Returns the underlying bytes representing the IRI reference as a mutable
	/// `Vec<u8>`.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid IRI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	pub fn path_mut(&mut self) -> PathMut {
		PathMut::from_impl(RiRefBufImpl::path_mut(self))
	}

	pub fn authority_mut(&mut self) -> Option<AuthorityMut> {
		RiRefBufImpl::authority_mut(self).map(AuthorityMut::from_impl)
	}

	/// Sets the scheme part.
	///
	/// If there is no authority and the start of the path looks like a scheme
	/// (e.g. `foo:`) then the path is prefixed with `./` to avoid being
	/// confused with a scheme.
	///
	/// # Example
	///
	/// ```
	/// use iref::{IriRefBuf, iri::Scheme};
	///
	/// let mut a = IriRefBuf::new("foo/bar".to_string()).unwrap();
	/// a.set_scheme(Some(Scheme::new(b"http").unwrap()));
	/// assert_eq!(a, "http:foo/bar");
	///
	/// let mut b = IriRefBuf::new("scheme://example.org/foo/bar".to_string()).unwrap();
	/// b.set_scheme(None);
	/// assert_eq!(b, "//example.org/foo/bar");
	///
	/// let mut c = IriRefBuf::new("scheme:foo:bar".to_string()).unwrap();
	/// c.set_scheme(None);
	/// assert_eq!(c, "./foo:bar");
	/// ```
	pub fn set_scheme(&mut self, scheme: Option<&Scheme>) {
		RiRefBufImpl::set_scheme(self, scheme)
	}

	/// Sets the authority part.
	///
	/// If the path is relative, this also turns it into an absolute path,
	/// since an authority cannot be followed by a relative path.
	///
	/// To avoid any ambiguity, if `authority` is `None` and the path starts
	/// with `//`, it will be changed into `/.//` as to not be interpreted as
	/// an authority part.
	///
	/// # Example
	///
	/// ```
	/// use iref::{IriRefBuf, iri::Authority};
	///
	/// let mut a = IriRefBuf::new("scheme:/path".to_string()).unwrap();
	/// a.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(a, "scheme://example.org/path");
	///
	/// // When an authority is added before a relative path,
	/// // the path becomes absolute.
	/// let mut b = IriRefBuf::new("scheme:path".to_string()).unwrap();
	/// b.set_authority(Some(Authority::new("example.org").unwrap()));
	/// assert_eq!(b, "scheme://example.org/path");
	///
	/// // When an authority is removed and the path starts with `//`,
	/// // a `/.` prefix is added to the path to avoid any ambiguity.
	/// let mut c = IriRefBuf::new("scheme://example.org//path".to_string()).unwrap();
	/// c.set_authority(None);
	/// assert_eq!(c, "scheme:/.//path");
	/// ```
	pub fn set_authority(&mut self, authority: Option<&Authority>) {
		RiRefBufImpl::set_authority(self, authority)
	}

	/// Sets the path part.
	///
	/// If there is an authority and the path is relative, this also turns it
	/// into an absolute path, since an authority cannot be followed by a
	/// relative path.
	///
	/// To avoid any ambiguity, if there is no authority and the path starts
	/// with `//`, it will be changed into `/.//` as to not be interpreted as
	/// an authority part. Similarly if there is no scheme nor authority and the
	/// beginning of the new path looks like a scheme, it is prefixed with `./`
	/// to not be confused with a scheme.
	///
	/// # Example
	///
	/// ```
	/// use iref::{IriRefBuf, iri::Path};
	///
	/// let mut a = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// a.set_path(Path::new("/foo/bar").unwrap());
	/// assert_eq!(a, "http://example.org/foo/bar");
	///
	/// // If there is an authority and the new path is relative,
	/// // it is turned into an absolute path.
	/// let mut b = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
	/// b.set_path(Path::new("relative/path").unwrap());
	/// assert_eq!(b, "http://example.org/relative/path");
	///
	/// // If there is no authority and the path starts with `//`,
	/// // it is prefixed with `/.` to avoid being confused with an authority.
	/// let mut c = IriRefBuf::new("http:old/path".to_string()).unwrap();
	/// c.set_path(Path::new("//foo/bar").unwrap());
	/// assert_eq!(c, "http:/.//foo/bar");
	///
	/// // If there is no authority nor scheme, and the path beginning looks
	/// // like a scheme, it is prefixed with `./` to avoid being confused with
	/// // a scheme.
	/// let mut d = IriRefBuf::new("old/path".to_string()).unwrap();
	/// d.set_path(Path::new("foo:bar").unwrap());
	/// assert_eq!(d, "./foo:bar");
	/// ```
	pub fn set_path(&mut self, path: &Path) {
		RiRefBufImpl::set_path(self, path)
	}

	/// Sets the query part.
	pub fn set_query(&mut self, query: Option<&Query>) {
		RiRefBufImpl::set_query(self, query)
	}

	/// Sets the fragment part.
	pub fn set_fragment(&mut self, fragment: Option<&Fragment>) {
		RiRefBufImpl::set_fragment(self, fragment)
	}

	/// Resolve the IRI reference.
	///
	/// ## Abnormal use of dot segments.
	///
	/// See <https://www.rfc-editor.org/errata/eid4547>
	pub fn resolve(&mut self, base_iri: &(impl ?Sized + AsRef<Iri>)) {
		RiRefBufImpl::resolve(self, base_iri.as_ref())
	}

	pub fn into_resolved(self, base_iri: &(impl ?Sized + AsRef<Iri>)) -> IriBuf {
		RiRefBufImpl::into_resolved(self, base_iri.as_ref())
	}
}

impl PartialEq<str> for IriRefBuf {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl<'a> PartialEq<&'a str> for IriRefBuf {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<String> for IriRefBuf {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other.as_str()
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
			("scheme:/path", Some("authority"), "scheme://authority/path"),
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
