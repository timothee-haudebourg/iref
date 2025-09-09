use std::hash::{Hash, Hasher};

use super::{Authority, AuthorityMut, Fragment, Path, PathBuf, PathMut, Query, Scheme, Segment};
use crate::{
	InvalidIri, InvalidUri, Iri, IriBuf, Uri, UriBuf, UriRef, UriRefBuf, uri::InvalidUriRef,
};

crate::common::reference!("IRI": Iri, IriBuf, IriRef, IriRefBuf);

/// Parses an [`IriRef`] at compile time.
#[macro_export]
macro_rules! iri_ref {
	($value:literal) => {
		match $crate::iri::IriRef::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI reference"),
		}
	};
}

impl IriRef {
	/// Converts this IRI reference into an IRI, if possible.
	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	/// Converts this IRI reference into an URI, if possible.
	pub fn as_uri(&self) -> Option<&Uri> {
		Uri::new(self.as_bytes()).ok()
	}

	/// Converts this IRI reference into an URI reference, if possible.
	pub fn as_uri_ref(&self) -> Option<&UriRef> {
		UriRef::new(self.as_bytes()).ok()
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a Iri {
	type Error = InvalidIri<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_iri().ok_or(InvalidIri(value))
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a Uri {
	type Error = InvalidUri<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a IriRef> for &'a UriRef {
	type Error = InvalidUriRef<&'a IriRef>;

	fn try_from(value: &'a IriRef) -> Result<Self, Self::Error> {
		value.as_uri_ref().ok_or(InvalidUriRef(value))
	}
}

impl PartialEq for IriRef {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a IriRef> for IriRef {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl PartialEq<Iri> for IriRef {
	fn eq(&self, other: &Iri) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a Iri> for IriRef {
	fn eq(&self, other: &&'a Iri) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl PartialEq<IriBuf> for IriRef {
	fn eq(&self, other: &IriBuf) -> bool {
		*self == *other.as_iri_ref()
	}
}

impl Eq for IriRef {}

impl PartialOrd for IriRef {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a IriRef> for IriRef {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<Iri> for IriRef {
	fn partial_cmp(&self, other: &Iri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl<'a> PartialOrd<&'a Iri> for IriRef {
	fn partial_cmp(&self, other: &&'a Iri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl PartialOrd<IriBuf> for IriRef {
	fn partial_cmp(&self, other: &IriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_iri_ref())
	}
}

impl Ord for IriRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for IriRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

// crate::common::owned_reference!(Iri, IriBuf, IriRef, IriRefBuf);

impl IriRefBuf {
	/// Creates a new IRI reference from a byte string.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<Self, InvalidIriRef<Vec<u8>>> {
		match String::from_utf8(buffer) {
			Ok(string) => {
				Self::new(string).map_err(|InvalidIriRef(s)| InvalidIriRef(s.into_bytes()))
			}
			Err(e) => Err(InvalidIriRef(e.into_bytes())),
		}
	}

	/// Creates a new IRI reference from a byte string without varidation.
	///
	/// # Safety
	///
	/// The input bytes must be a valid IRI reference.
	#[inline]
	pub unsafe fn from_vec_unchecked(buffer: Vec<u8>) -> Self {
		unsafe { Self::new_unchecked(String::from_utf8_unchecked(buffer)) }
	}

	/// Converts this IRI reference into an IRI, if possible.
	pub fn try_into_iri(self) -> Result<IriBuf, InvalidIri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(IriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidIri(self))
		}
	}

	/// Converts this IRI reference into an URI, if possible.
	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<Self>> {
		UriBuf::new(self.into_bytes()).map_err(|InvalidUri(bytes)| unsafe {
			InvalidUri(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Converts this IRI reference into an URI reference, if possible.
	pub fn try_into_uri_ref(self) -> Result<UriRefBuf, InvalidUriRef<Self>> {
		UriRefBuf::new(self.into_bytes()).map_err(|InvalidUriRef(bytes)| unsafe {
			InvalidUriRef(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Returns the underlying bytes representing the IRI reference as a mutable
	/// `Vec<u8>`.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid IRI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		unsafe { self.0.as_mut_vec() }
	}
}

impl TryFrom<IriRefBuf> for IriBuf {
	type Error = InvalidIri<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_iri()
	}
}

impl TryFrom<IriRefBuf> for UriBuf {
	type Error = InvalidUri<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

impl TryFrom<IriRefBuf> for UriRefBuf {
	type Error = InvalidUriRef<IriRefBuf>;

	fn try_from(value: IriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri_ref()
	}
}

impl PartialEq<Iri> for IriRefBuf {
	fn eq(&self, other: &Iri) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a Iri> for IriRefBuf {
	fn eq(&self, other: &&'a Iri) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl PartialEq<IriBuf> for IriRefBuf {
	fn eq(&self, other: &IriBuf) -> bool {
		*self.as_iri_ref() == *other.as_iri_ref()
	}
}

impl PartialOrd<Iri> for IriRefBuf {
	fn partial_cmp(&self, other: &Iri) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
	}
}

impl<'a> PartialOrd<&'a Iri> for IriRefBuf {
	fn partial_cmp(&self, other: &&'a Iri) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
	}
}

impl PartialOrd<IriBuf> for IriRefBuf {
	fn partial_cmp(&self, other: &IriBuf) -> Option<std::cmp::Ordering> {
		self.as_iri_ref().partial_cmp(other.as_iri_ref())
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
			// eprintln!("{input}: {expected:?}");
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
			// eprintln!("{input}, {authority:?} => {buffer}, {expected}");
			assert_eq!(buffer.as_str(), expected)
		}
	}

	#[test]
	fn path() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.path().as_str(), expected.2)
		}
	}

	#[test]
	fn query() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.query().map(Query::as_str), expected.3)
		}
	}

	#[test]
	fn fragment() {
		for (input, expected) in PARTS {
			let input = IriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.fragment().map(Fragment::as_str), expected.4)
		}
	}

	#[test]
	fn disambiguate_scheme() {
		let mut iri_ref = IriRefBuf::new("scheme:a:b/c".to_string()).unwrap();
		iri_ref.set_scheme(None);
		assert_eq!(iri_ref.as_str(), "./a:b/c")
	}

	#[test]
	fn disambiguate_authority() {
		let mut iri_ref = IriRefBuf::new("//host//path".to_string()).unwrap();
		iri_ref.set_authority(None);
		assert_eq!(iri_ref.as_str(), "/.//path")
	}

	#[test]
	fn unambiguous_resolution() {
		let base_iri = Iri::new("http:/a/b").unwrap();
		let tests = [("../..//", "http:/")];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_normal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("g:h", "g:h"),
			("g", "http://a/b/c/g"),
			("./g", "http://a/b/c/g"),
			("g/", "http://a/b/c/g/"),
			("/g", "http://a/g"),
			("//g", "http://g"),
			("?y", "http://a/b/c/d;p?y"),
			("g?y", "http://a/b/c/g?y"),
			("#s", "http://a/b/c/d;p?q#s"),
			("g#s", "http://a/b/c/g#s"),
			("g?y#s", "http://a/b/c/g?y#s"),
			(";x", "http://a/b/c/;x"),
			("g;x", "http://a/b/c/g;x"),
			("g;x?y#s", "http://a/b/c/g;x?y#s"),
			("", "http://a/b/c/d;p?q"),
			(".", "http://a/b/c/"),
			("./", "http://a/b/c/"),
			("..", "http://a/b/"),
			("../", "http://a/b/"),
			("../g", "http://a/b/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g"),
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_abnormal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		// NOTE we implement [Errata 4547](https://www.rfc-editor.org/errata/eid4547)
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("../../../g", "http://a/g"),
			("../../../../g", "http://a/g"),
			("/./g", "http://a/g"),
			("/../g", "http://a/g"),
			("g.", "http://a/b/c/g."),
			(".g", "http://a/b/c/.g"),
			("g..", "http://a/b/c/g.."),
			("..g", "http://a/b/c/..g"),
			("./../g", "http://a/b/g"),
			("./g/.", "http://a/b/c/g/"),
			("g/./h", "http://a/b/c/g/h"),
			("g/../h", "http://a/b/c/h"),
			("g;x=1/./y", "http://a/b/c/g;x=1/y"),
			("g;x=1/../y", "http://a/b/c/y"),
			("g?y/./x", "http://a/b/c/g?y/./x"),
			("g?y/../x", "http://a/b/c/g?y/../x"),
			("g#s/./x", "http://a/b/c/g#s/./x"),
			("g#s/../x", "http://a/b/c/g#s/../x"),
			("http:g", "http:g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn more_resolutions1() {
		let base_iri = Iri::new("http://a/bb/ccc/d;p?q").unwrap();

		let tests = [
			("#s", "http://a/bb/ccc/d;p?q#s"),
			("", "http://a/bb/ccc/d;p?q"),
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions2() {
		let base_iri = Iri::new("http://a/bb/ccc/./d;p?q").unwrap();

		let tests = [
			("..", "http://a/bb/"),
			("../", "http://a/bb/"),
			("../g", "http://a/bb/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions3() {
		let base_iri = Iri::new("http://ab//de//ghi").unwrap();

		let tests = [
			("xyz", "http://ab//de//xyz"),
			("./xyz", "http://ab//de//xyz"),
			("../xyz", "http://ab//de/xyz"),
		];

		for (relative, absolute) in &tests {
			println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions4() {
		let base_iri = Iri::new("http://a/bb/ccc/../d;p?q").unwrap();

		let tests = [("../../", "http://a/")];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	// https://github.com/timothee-haudebourg/iref/issues/14
	#[test]
	fn reference_resolution_with_scheme_no_disambiguation() {
		let base = Iri::new("scheme:a:b/").unwrap();
		let mut iri = IriRefBuf::new("Foo".to_string()).unwrap();
		iri.resolve(base);

		assert_eq!(iri.to_string(), "scheme:a:b/Foo")
	}

	#[test]
	fn relative_to() {
		let base =
			IriRef::new("https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld").unwrap();
		let vectors = [
			(
				"https://w3c.github.io/json-ld-api/tests/compact/link",
				"link",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld#fragment-works",
				"#fragment-works",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld?query=works",
				"?query=works",
			),
			("https://w3c.github.io/json-ld-api/tests/", "../"),
			("https://w3c.github.io/json-ld-api/", "../../"),
			("https://w3c.github.io/json-ld-api/parent", "../../parent"),
			(
				"https://w3c.github.io/json-ld-api/parent#fragment",
				"../../parent#fragment",
			),
			(
				"https://w3c.github.io/parent-parent-eq-root",
				"../../../parent-parent-eq-root",
			),
			(
				"http://example.org/scheme-relative",
				"http://example.org/scheme-relative",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld",
				"0066-in.jsonld",
			),
		];

		for (input, expected) in &vectors {
			let input = IriRef::new(input).unwrap();
			assert_eq!(input.relative_to(base), *expected)
		}
	}
}
