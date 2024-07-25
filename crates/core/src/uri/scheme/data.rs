use std::{
	borrow::{Borrow, Cow},
	ops::Deref,
	str::FromStr,
};

use base64::Engine;

use crate::{InvalidUri, Uri, UriBuf};

#[derive(Debug, thiserror::Error)]
#[error("invalid data URL `{0}`")]
pub struct InvalidDataUrl<T = String>(pub T);

impl<T> From<InvalidUri<T>> for InvalidDataUrl<T> {
	fn from(value: InvalidUri<T>) -> Self {
		Self(value.0)
	}
}

impl<'a, T: ?Sized + ToOwned> InvalidDataUrl<&'a T> {
	pub fn into_owned(self) -> InvalidDataUrl<T::Owned> {
		InvalidDataUrl(self.0.to_owned())
	}
}

/// Data URL.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DataUrl(Uri);

impl DataUrl {
	/// Creates a new data URL by parsing the given URL.
	pub fn new<T>(url: &T) -> Result<&Self, InvalidDataUrl<&T>>
	where
		T: ?Sized + AsRef<[u8]>,
	{
		let uri = Uri::new(url.as_ref()).map_err(|_| InvalidDataUrl(url))?;
		if DataUrlDelimiters::parse(uri).is_some() {
			Ok(unsafe { Self::new_unchecked(uri) })
		} else {
			Err(InvalidDataUrl(url))
		}
	}

	/// Creates a new data URL from the given input without validation.
	///
	/// # Safety
	///
	/// The input value must be a data URL.
	pub unsafe fn new_unchecked(url: &(impl ?Sized + AsRef<[u8]>)) -> &Self {
		std::mem::transmute(url.as_ref())
	}

	pub fn parts(&self) -> DataUrlPartsRef {
		DataUrlPartsRef::parse(&self.0).unwrap()
	}

	pub fn media_type(&self) -> Option<&str> {
		let mut chars = self.0.char_indices();
		loop {
			if let Some((i, ';' | ',')) = chars.next() {
				break non_empty(&self.0[5..i]);
			}
		}
	}

	pub fn is_base_64_encoded(&self) -> bool {
		let mut chars = self.0.chars();
		loop {
			match chars.next() {
				Some(',') => break false,
				Some(';') => break true,
				_ => (),
			}
		}
	}

	pub fn encoded_data(&self) -> &str {
		let mut chars = self.0.char_indices();
		loop {
			if let Some((i, ',')) = chars.next() {
				break &self.0[(i + 1)..];
			}
		}
	}

	pub fn decoded_data(&self) -> Result<Cow<[u8]>, base64::DecodeError> {
		let encoded = self.encoded_data();
		if self.is_base_64_encoded() {
			base64::engine::general_purpose::STANDARD
				.decode(encoded)
				.map(Cow::Owned)
		} else {
			Ok(Cow::Borrowed(encoded.as_bytes()))
		}
	}

	pub fn as_uri(&self) -> &Uri {
		&self.0
	}

	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl Deref for DataUrl {
	type Target = Uri;

	fn deref(&self) -> &Self::Target {
		self.as_uri()
	}
}

impl AsRef<DataUrl> for DataUrl {
	fn as_ref(&self) -> &DataUrl {
		self
	}
}

impl AsRef<Uri> for DataUrl {
	fn as_ref(&self) -> &Uri {
		&self.0
	}
}

impl<'a> TryFrom<&'a str> for &'a DataUrl {
	type Error = InvalidDataUrl;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		DataUrl::new(value).map_err(InvalidDataUrl::into_owned)
	}
}

/// Owned data URL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataUrlBuf {
	url: UriBuf,
	delimiters: DataUrlDelimiters,
}

impl DataUrlBuf {
	/// Creates a new data URL by parsing the given bytes.
	pub fn new(url: impl Into<Vec<u8>>) -> Result<Self, InvalidDataUrl<Vec<u8>>> {
		let url = UriBuf::new(url.into())?;
		match DataUrlDelimiters::parse(&url) {
			Some(delimiters) => Ok(Self { url, delimiters }),
			None => Err(InvalidDataUrl(url.into_bytes())),
		}
	}

	/// Creates a new data URL by parsing the given string.
	pub fn from_string(url: String) -> Result<Self, InvalidDataUrl> {
		Self::new(url).map_err(|InvalidDataUrl(bytes)| {
			InvalidDataUrl(unsafe { String::from_utf8_unchecked(bytes) })
		})
	}

	/// Creates a new data URL from the given input without validation.
	///
	/// # Safety
	///
	/// The input value must be a data URL.
	pub unsafe fn new_unchecked(url: impl Into<Vec<u8>>) -> Self {
		let url = UriBuf::new_unchecked(url.into());
		let delimiters = DataUrlDelimiters::parse(&url).unwrap();
		Self { url, delimiters }
	}

	pub fn parts(&self) -> DataUrlPartsRef {
		self.delimiters.into_parts(&self.url)
	}

	pub fn media_type(&self) -> Option<&str> {
		self.delimiters.media_type(&self.url)
	}

	pub fn is_base_64_encoded(&self) -> bool {
		self.delimiters.base_64
	}

	pub fn encoded_data(&self) -> &str {
		self.delimiters.data(&self.url)
	}

	pub fn decoded_data(&self) -> Result<Cow<[u8]>, base64::DecodeError> {
		let encoded = self.encoded_data();
		if self.is_base_64_encoded() {
			base64::engine::general_purpose::STANDARD
				.decode(encoded)
				.map(Cow::Owned)
		} else {
			Ok(Cow::Borrowed(encoded.as_bytes()))
		}
	}

	pub fn as_data_url(&self) -> &DataUrl {
		unsafe { DataUrl::new_unchecked(&self.url) }
	}
}

impl Deref for DataUrlBuf {
	type Target = DataUrl;

	fn deref(&self) -> &Self::Target {
		self.as_data_url()
	}
}

impl Borrow<DataUrl> for DataUrlBuf {
	fn borrow(&self) -> &DataUrl {
		self.as_data_url()
	}
}

impl AsRef<DataUrl> for DataUrlBuf {
	fn as_ref(&self) -> &DataUrl {
		self.as_data_url()
	}
}

impl AsRef<Uri> for DataUrlBuf {
	fn as_ref(&self) -> &Uri {
		&self.0
	}
}

impl TryFrom<String> for DataUrlBuf {
	type Error = InvalidDataUrl;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::from_string(value)
	}
}

impl FromStr for DataUrlBuf {
	type Err = InvalidDataUrl;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::from_string(s.to_owned())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DataUrlDelimiters {
	media_type_end: usize,
	base_64: bool,
	data_start: usize,
}

impl DataUrlDelimiters {
	fn parse(url: &str) -> Option<Self> {
		let suffix = url.strip_prefix("data:")?;
		let mut chars = suffix.char_indices();

		// Parse media type.
		loop {
			match chars.next() {
				Some((i, ',')) => {
					// no base64
					break Some(Self {
						media_type_end: 5 + i,
						base_64: false,
						data_start: 5 + i + 1,
					});
				}
				Some((i, ';')) => {
					// base64
					let j = i + 8;
					break if suffix.len() >= j && &suffix[(i + 1)..j] == "base64," {
						Some(Self {
							media_type_end: 5 + i,
							base_64: true,
							data_start: 5 + j,
						})
					} else {
						None
					};
				}
				Some((_, c)) if is_media_type_char(c) => (),
				_ => break None,
			}
		}
	}

	fn media_type<'a>(&self, url: &'a str) -> Option<&'a str> {
		non_empty(&url[5..self.media_type_end])
	}

	fn data<'a>(&self, url: &'a str) -> &'a str {
		&url[self.data_start..]
	}

	fn into_parts(self, url: &str) -> DataUrlPartsRef {
		DataUrlPartsRef {
			media_type: self.media_type(url),
			base_64: self.base_64,
			data: self.data(url),
		}
	}
}

/// Data URL parts references.
#[derive(Debug, PartialEq, Eq)]
pub struct DataUrlPartsRef<'a> {
	pub media_type: Option<&'a str>,
	pub base_64: bool,
	pub data: &'a str,
}

impl<'a> DataUrlPartsRef<'a> {
	pub fn parse(url: &'a str) -> Option<Self> {
		Some(DataUrlDelimiters::parse(url)?.into_parts(url))
	}
}

fn is_media_type_char(c: char) -> bool {
	c.is_ascii_alphanumeric()
		|| matches!(c, '/' | '!' | '#' | '$' | '&' | '-' | '+' | '^' | '_' | '.')
}

fn non_empty(s: &str) -> Option<&str> {
	if s.is_empty() {
		None
	} else {
		Some(s)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for DataUrl {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for &'de DataUrl {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<&'de str>::deserialize(deserializer)?
			.try_into()
			.map_err(serde::de::Error::custom)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for DataUrlBuf {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_data_url().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DataUrlBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		String::deserialize(deserializer)?
			.try_into()
			.map_err(serde::de::Error::custom)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_data_url_1() {
		assert!(DataUrl::new("data:invalid").is_err())
	}

	#[test]
	fn parse_data_url_2() {
		let found = DataUrl::new("data:,valid").unwrap();
		let expected = DataUrlPartsRef {
			media_type: None,
			base_64: false,
			data: "valid",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), None);
		assert_eq!(found.is_base_64_encoded(), false);
		assert_eq!(found.encoded_data(), "valid");
	}

	#[test]
	fn parse_data_url_3() {
		let found = DataUrl::new("data:;base64,").unwrap();
		let expected = DataUrlPartsRef {
			media_type: None,
			base_64: true,
			data: "",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), None);
		assert_eq!(found.is_base_64_encoded(), true);
		assert_eq!(found.encoded_data(), "")
	}

	#[test]
	fn parse_data_url_4() {
		let found = DataUrl::new("data:;base64,data").unwrap();
		let expected = DataUrlPartsRef {
			media_type: None,
			base_64: true,
			data: "data",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), None);
		assert_eq!(found.is_base_64_encoded(), true);
		assert_eq!(found.encoded_data(), "data")
	}

	#[test]
	fn parse_data_url_5() {
		let found = DataUrl::new("data:image/jpeg,data").unwrap();
		let expected = DataUrlPartsRef {
			media_type: Some("image/jpeg"),
			base_64: false,
			data: "data",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), Some("image/jpeg"));
		assert_eq!(found.is_base_64_encoded(), false);
		assert_eq!(found.encoded_data(), "data")
	}

	#[test]
	fn parse_data_url_6() {
		let found = DataUrl::new("data:image/jpeg;base64,data").unwrap();
		let expected = DataUrlPartsRef {
			media_type: Some("image/jpeg"),
			base_64: true,
			data: "data",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), Some("image/jpeg"));
		assert_eq!(found.is_base_64_encoded(), true);
		assert_eq!(found.encoded_data(), "data")
	}

	#[test]
	fn parse_data_url_7() {
		let found = DataUrl::new("data:image/jpeg;base64,").unwrap();
		let expected = DataUrlPartsRef {
			media_type: Some("image/jpeg"),
			base_64: true,
			data: "",
		};

		assert_eq!(found.parts(), expected);
		assert_eq!(found.media_type(), Some("image/jpeg"));
		assert_eq!(found.is_base_64_encoded(), true);
		assert_eq!(found.encoded_data(), "")
	}
}
