use std::hash::{self, Hash};

use crate::{InvalidIri, InvalidUri, Iri, IriBuf, IriRef, IriRefBuf, Uri, UriBuf};

use super::{
	Authority, AuthorityMut, Fragment, InvalidAuthority, InvalidFragment, InvalidPath,
	InvalidQuery, Path, PathBuf, PathMut, Query, Scheme, Segment,
};

crate::common::reference!("URI": Uri, UriBuf, UriRef, UriRefBuf);

/// Parses an [`UriRef`] at compile time.
#[macro_export]
macro_rules! uri_ref {
	($value:literal) => {
		match $crate::uri::UriRef::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI reference"),
		}
	};
}

impl UriRef {
	#[inline]
	pub fn as_uri(&self) -> Option<&Uri> {
		if self.scheme().is_some() {
			Some(unsafe { Uri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(&self.0) })
		} else {
			None
		}
	}

	#[inline]
	pub const fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(&self.0) }
	}
}

impl AsRef<IriRef> for UriRef {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl<'a> From<&'a UriRef> for &'a IriRef {
	fn from(value: &'a UriRef) -> Self {
		value.as_iri_ref()
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Uri {
	type Error = InvalidUri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Iri {
	type Error = InvalidIri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_iri().ok_or(InvalidIri(value))
	}
}

impl PartialEq for UriRef {
	fn eq(&self, other: &Self) -> bool {
		self.parts() == other.parts()
	}
}

impl<'a> PartialEq<&'a UriRef> for UriRef {
	fn eq(&self, other: &&'a Self) -> bool {
		*self == **other
	}
}

impl PartialEq<Uri> for UriRef {
	fn eq(&self, other: &Uri) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl<'a> PartialEq<&'a Uri> for UriRef {
	fn eq(&self, other: &&'a Uri) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl PartialEq<UriBuf> for UriRef {
	fn eq(&self, other: &UriBuf) -> bool {
		*self == *other.as_uri_ref()
	}
}

impl Eq for UriRef {}

impl PartialOrd for UriRef {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialOrd<&'a UriRef> for UriRef {
	fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
		self.partial_cmp(*other)
	}
}

impl PartialOrd<Uri> for UriRef {
	fn partial_cmp(&self, other: &Uri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl<'a> PartialOrd<&'a Uri> for UriRef {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl PartialOrd<UriBuf> for UriRef {
	fn partial_cmp(&self, other: &UriBuf) -> Option<std::cmp::Ordering> {
		self.partial_cmp(other.as_uri_ref())
	}
}

impl Ord for UriRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl Hash for UriRef {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.parts().hash(state)
	}
}

impl UriRefBuf {
	/// Returns a mutable reference to the underlying `Vec<u8>` buffer
	/// representing the URI reference.
	///
	/// # Safety
	///
	/// The caller must ensure that once the mutable reference is dropped, its
	/// content is still a valid URI reference.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		unsafe { self.0.as_mut_vec() }
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(self) }
	}

	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(UriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidUri(self))
		}
	}

	pub fn try_into_iri(self) -> Result<IriBuf, InvalidIri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(IriBuf::new_unchecked(self.0)) }
		} else {
			Err(InvalidIri(self))
		}
	}
}

impl AsRef<IriRef> for UriRefBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl From<UriRefBuf> for IriRefBuf {
	fn from(value: UriRefBuf) -> Self {
		value.into_iri_ref()
	}
}

impl TryFrom<UriRefBuf> for UriBuf {
	type Error = InvalidUri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

impl TryFrom<UriRefBuf> for IriBuf {
	type Error = InvalidIri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_iri()
	}
}

impl PartialEq<Uri> for UriRefBuf {
	fn eq(&self, other: &Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl<'a> PartialEq<&'a Uri> for UriRefBuf {
	fn eq(&self, other: &&'a Uri) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl PartialEq<UriBuf> for UriRefBuf {
	fn eq(&self, other: &UriBuf) -> bool {
		*self.as_uri_ref() == *other.as_uri_ref()
	}
}

impl PartialOrd<Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &Uri) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

impl<'a> PartialOrd<&'a Uri> for UriRefBuf {
	fn partial_cmp(&self, other: &&'a Uri) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

impl PartialOrd<UriBuf> for UriRefBuf {
	fn partial_cmp(&self, other: &UriBuf) -> Option<std::cmp::Ordering> {
		self.as_uri_ref().partial_cmp(other.as_uri_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PARTS: [(
		&[u8],
		(
			Option<&[u8]>,
			Option<&[u8]>,
			&[u8],
			Option<&[u8]>,
			Option<&[u8]>,
		),
	); 36] = [
		// 0 components.
		(b"", (None, None, b"", None, None)),
		// 1 component.
		(b"scheme:", (Some(b"scheme"), None, b"", None, None)),
		(b"//authority", (None, Some(b"authority"), b"", None, None)),
		(b"path", (None, None, b"path", None, None)),
		(b"/path", (None, None, b"/path", None, None)),
		(b"/", (None, None, b"/", None, None)),
		(b"foo//bar", (None, None, b"foo//bar", None, None)),
		(b"?query", (None, None, b"", Some(b"query"), None)),
		(b"#fragment", (None, None, b"", None, Some(b"fragment"))),
		(
			b"scheme:?query",
			(Some(b"scheme"), None, b"", Some(b"query"), None),
		),
		// 2 components.
		(
			b"scheme://authority",
			(Some(b"scheme"), Some(b"authority"), b"", None, None),
		),
		(b"scheme:path", (Some(b"scheme"), None, b"path", None, None)),
		(
			b"scheme:/path",
			(Some(b"scheme"), None, b"/path", None, None),
		),
		(
			b"scheme:?query",
			(Some(b"scheme"), None, b"", Some(b"query"), None),
		),
		(
			b"scheme:#fragment",
			(Some(b"scheme"), None, b"", None, Some(b"fragment")),
		),
		(
			b"//authority/path",
			(None, Some(b"authority"), b"/path", None, None),
		),
		(
			b"//authority?query",
			(None, Some(b"authority"), b"", Some(b"query"), None),
		),
		(
			b"//authority#fragment",
			(None, Some(b"authority"), b"", None, Some(b"fragment")),
		),
		(b"path?query", (None, None, b"path", Some(b"query"), None)),
		(b"/path?query", (None, None, b"/path", Some(b"query"), None)),
		(
			b"path#fragment",
			(None, None, b"path", None, Some(b"fragment")),
		),
		(
			b"?query#fragment",
			(None, None, b"", Some(b"query"), Some(b"fragment")),
		),
		// 3 components
		(
			b"scheme://authority/path",
			(Some(b"scheme"), Some(b"authority"), b"/path", None, None),
		),
		(
			b"scheme://authority?query",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				Some(b"query"),
				None,
			),
		),
		(
			b"scheme://authority#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				None,
				Some(b"fragment"),
			),
		),
		(
			b"scheme:path?query",
			(Some(b"scheme"), None, b"path", Some(b"query"), None),
		),
		(
			b"scheme:path#fragment",
			(Some(b"scheme"), None, b"path", None, Some(b"fragment")),
		),
		(
			b"//authority/path?query",
			(None, Some(b"authority"), b"/path", Some(b"query"), None),
		),
		(
			b"//authority/path#fragment",
			(None, Some(b"authority"), b"/path", None, Some(b"fragment")),
		),
		(
			b"//authority?query#fragment",
			(
				None,
				Some(b"authority"),
				b"",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		(
			b"path?query#fragment",
			(None, None, b"path", Some(b"query"), Some(b"fragment")),
		),
		// 4 components
		(
			b"scheme://authority/path?query",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				Some(b"query"),
				None,
			),
		),
		(
			b"scheme://authority/path#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				None,
				Some(b"fragment"),
			),
		),
		(
			b"scheme://authority?query#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		(
			b"scheme:path?query#fragment",
			(
				Some(b"scheme"),
				None,
				b"path",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
		// 5 components
		(
			b"scheme://authority/path?query#fragment",
			(
				Some(b"scheme"),
				Some(b"authority"),
				b"/path",
				Some(b"query"),
				Some(b"fragment"),
			),
		),
	];

	#[test]
	fn parts() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.scheme.map(Scheme::as_bytes), expected.0);
			assert_eq!(parts.authority.map(Authority::as_bytes), expected.1);
			assert_eq!(parts.path.as_bytes(), expected.2);
			assert_eq!(parts.query.map(Query::as_bytes), expected.3);
			assert_eq!(parts.fragment.map(Fragment::as_bytes), expected.4)
		}
	}

	#[test]
	fn scheme() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			assert_eq!(input.scheme().map(Scheme::as_bytes), expected.0)
		}
	}

	#[test]
	fn authority() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.authority().map(Authority::as_bytes), expected.1)
		}
	}

	#[test]
	fn set_authority() {
		let vectors: [(&[u8], Option<&[u8]>, &[u8]); 3] = [
			(
				b"scheme:/path",
				Some(b"authority"),
				b"scheme://authority/path",
			),
			(
				b"scheme:path",
				Some(b"authority"),
				b"scheme://authority/path",
			),
			(b"scheme://authority//path", None, b"scheme:/.//path"),
		];

		for (input, authority, expected) in vectors {
			let mut buffer = UriRefBuf::new(input.to_vec()).unwrap();
			let authority = authority.map(Authority::new).transpose().unwrap();
			buffer.set_authority(authority);
			// eprintln!("{input:?}, {authority:?} => {buffer}, {expected:?}");
			assert_eq!(buffer.as_bytes(), expected)
		}
	}

	#[test]
	fn path() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.path().as_bytes(), expected.2)
		}
	}

	#[test]
	fn query() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.query().map(Query::as_bytes), expected.3)
		}
	}

	#[test]
	fn fragment() {
		for (input, expected) in PARTS {
			let input = UriRef::new(input).unwrap();
			// eprintln!("{input}: {expected:?}");
			assert_eq!(input.fragment().map(Fragment::as_bytes), expected.4)
		}
	}
}
