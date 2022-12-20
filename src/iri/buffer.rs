use std::{
	borrow::Borrow,
	cmp::{Ord, Ordering, PartialOrd},
	convert::TryFrom,
	fmt,
	hash::{Hash, Hasher},
	ops::Deref,
	str::FromStr,
};

use crate::{
	iri::Iri, parsing::ParsedIriRef, AsIri, AsIriRef, Authority, AuthorityMut, Error, Fragment,
	IriRef, IriRefBuf, Path, PathMut, Query, Scheme,
};

/// Owned IRI.
#[derive(Clone)]
pub struct IriBuf(pub(crate) IriRefBuf);

impl IriBuf {
	/// Creates a new IRI reference by parsing and copying the input buffer.
	#[inline]
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<Self, Error> {
		let iri_ref = IriRefBuf::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(Self(iri_ref))
		} else {
			Err(Error::MissingScheme)
		}
	}

	/// Creates a new IRI by parsing and the input buffer.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
		let iri_ref = IriRefBuf::from_vec(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(Self(iri_ref))
		} else {
			Err((Error::MissingScheme, iri_ref.into_bytes()))
		}
	}

	/// Creates a new IRI by parsing and the input string buffer.
	#[inline]
	pub fn from_string(buffer: String) -> Result<Self, (Error, String)> {
		let iri_ref = IriRefBuf::from_string(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(Self(iri_ref))
		} else {
			Err(unsafe {
				let mut vec = iri_ref.into_bytes();
				let ptr = vec.as_mut_ptr();
				let len = vec.len();
				let capacity = vec.capacity();
				std::mem::forget(vec);
				(
					Error::MissingScheme,
					String::from_raw_parts(ptr, len, capacity),
				)
			})
		}
	}

	/// Consume the IRI and return its constituting parts:
	/// the internal buffer and parsing data.
	#[inline]
	pub fn into_raw_parts(self) -> (Vec<u8>, ParsedIriRef) {
		self.0.into_raw_parts()
	}

	/// Consume the IRI and return its internal buffer.
	#[inline]
	pub fn into_bytes(self) -> Vec<u8> {
		self.0.into_bytes()
	}

	/// Consume the IRI and return its internal buffer as a string.
	#[inline]
	pub fn into_string(self) -> String {
		self.0.into_string()
	}

	/// Creates a new IRI using `buffer` and the parsing information `p`.
	/// The parsing information is not checked against `buffer`.
	///
	/// ## Safety
	///
	/// The parsed data must match the given `buffer`.
	/// The scheme must not be empty.
	#[inline]
	pub unsafe fn from_raw_parts(buffer: Vec<u8>, p: ParsedIriRef) -> Self {
		let iri_ref = IriRefBuf::from_raw_parts(buffer, p);
		Self(iri_ref)
	}

	#[inline]
	pub fn from_scheme(scheme: Scheme) -> Self {
		let mut iri_ref = IriRefBuf::default();
		iri_ref.set_scheme(Some(scheme));
		IriBuf(iri_ref)
	}

	#[inline]
	pub fn as_iri(&self) -> Iri {
		Iri(self.0.as_iri_ref())
	}

	#[inline]
	pub fn as_iri_ref(&self) -> IriRef {
		self.0.as_iri_ref()
	}

	#[inline]
	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}

	/// Set the scheme of the IRI.
	#[inline]
	pub fn set_scheme(&mut self, scheme: Scheme) {
		self.0.set_scheme(Some(scheme))
	}

	#[inline]
	pub fn authority_mut(&mut self) -> Option<AuthorityMut> {
		self.0.authority_mut()
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	#[inline]
	pub fn set_authority(&mut self, authority: Option<Authority>) {
		self.0.set_authority(authority)
	}

	#[inline]
	pub fn path_mut(&mut self) -> PathMut {
		self.0.path_mut()
	}

	/// Set the IRI path.
	#[inline]
	pub fn set_path(&mut self, path: Path) {
		self.0.set_path(path)
	}

	#[inline]
	pub fn set_query(&mut self, query: Option<Query>) {
		self.0.set_query(query)
	}

	#[inline]
	pub fn set_fragment(&mut self, fragment: Option<Fragment>) {
		self.0.set_fragment(fragment)
	}
}

impl TryFrom<Vec<u8>> for IriBuf {
	type Error = (Error, Vec<u8>);

	fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
		Self::from_vec(v)
	}
}

impl TryFrom<String> for IriBuf {
	type Error = (Error, String);

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Self::from_string(s)
	}
}

impl FromStr for IriBuf {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s)
	}
}

impl AsIri for IriBuf {
	#[inline]
	fn as_iri(&self) -> Iri {
		self.as_iri()
	}
}

impl AsIriRef for IriBuf {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		self.as_iri_ref()
	}
}

impl Deref for IriBuf {
	type Target = IriRefBuf;

	#[inline]
	fn deref(&self) -> &IriRefBuf {
		&self.0
	}
}

impl fmt::Display for IriBuf {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl fmt::Debug for IriBuf {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl PartialEq for IriBuf {
	#[inline]
	fn eq(&self, other: &IriBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl Eq for IriBuf {}

impl<'a> PartialEq<Iri<'a>> for IriBuf {
	#[inline]
	fn eq(&self, other: &Iri<'a>) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<IriRef<'a>> for IriBuf {
	#[inline]
	fn eq(&self, other: &IriRef<'a>) -> bool {
		self.as_iri_ref() == *other
	}
}

impl PartialEq<IriRefBuf> for IriBuf {
	#[inline]
	fn eq(&self, other: &IriRefBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl<'a> PartialEq<&'a str> for IriBuf {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Iri::new(other) {
			self == &other
		} else {
			false
		}
	}
}

impl PartialOrd for IriBuf {
	#[inline]
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl Ord for IriBuf {
	#[inline]
	fn cmp(&self, other: &IriBuf) -> Ordering {
		self.as_iri_ref().cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<Iri<'a>> for IriBuf {
	#[inline]
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriRef<'a>> for IriBuf {
	#[inline]
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl PartialOrd<IriRefBuf> for IriBuf {
	#[inline]
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<Iri<'a>> for IriBuf {
	#[inline]
	fn from(iri: Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl<'a> From<&'a Iri<'a>> for IriBuf {
	#[inline]
	fn from(iri: &'a Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl<'a> TryFrom<IriRef<'a>> for IriBuf {
	type Error = Error;

	#[inline]
	fn try_from(iri_ref: IriRef<'a>) -> Result<IriBuf, Error> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(IriBuf(iri_ref.into()))
		} else {
			Err(Error::InvalidScheme)
		}
	}
}

impl TryFrom<IriRefBuf> for IriBuf {
	type Error = IriRefBuf;

	#[inline]
	fn try_from(iri_ref: IriRefBuf) -> Result<IriBuf, IriRefBuf> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(iri_ref)
		}
	}
}

impl Hash for IriBuf {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}

impl AsRef<str> for IriBuf {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for IriBuf {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Borrow<str> for IriBuf {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<[u8]> for IriBuf {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IriBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = IriBuf;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "an IRI")
			}

			fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				IriBuf::new(v).map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(v), &self))
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				IriBuf::from_str(v)
					.map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
			}

			fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				IriBuf::from_vec(v).map_err(|(_, b)| {
					E::invalid_value(serde::de::Unexpected::Bytes(b.as_slice()), &self)
				})
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				IriBuf::from_string(v).map_err(|(_, s)| {
					E::invalid_value(serde::de::Unexpected::Str(s.as_str()), &self)
				})
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for IriBuf {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}
