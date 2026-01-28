use crate::{InvalidUri, Iri, IriRef, Uri, UriRef, uri::InvalidUriRef};

#[cfg(feature = "std")]
use crate::{IriBuf, IriRefBuf, UriBuf, UriRefBuf};

impl Iri {
	/// Converts this IRI into an URI, if possible.
	pub fn as_uri(&self) -> Option<&Uri> {
		Uri::new(self.as_bytes()).ok()
	}

	/// Converts this IRI into an URI reference, if possible.
	pub fn as_uri_ref(&self) -> Option<&UriRef> {
		UriRef::new(self.as_bytes()).ok()
	}
}

impl<'a> TryFrom<&'a Iri> for &'a Uri {
	type Error = InvalidUri<&'a Iri>;

	fn try_from(value: &'a Iri) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a Iri> for &'a UriRef {
	type Error = InvalidUriRef<&'a Iri>;

	fn try_from(value: &'a Iri) -> Result<Self, Self::Error> {
		value.as_uri_ref().ok_or(InvalidUriRef(value))
	}
}

#[cfg(feature = "std")]
impl IriBuf {
	/// Converts this IRI into an URI, if possible.
	pub fn try_into_uri(self) -> Result<UriBuf, InvalidUri<IriBuf>> {
		UriBuf::new(self.into_bytes()).map_err(|InvalidUri(bytes)| unsafe {
			InvalidUri(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}

	/// Converts this IRI into an URI reference, if possible.
	pub fn try_into_uri_ref(self) -> Result<UriRefBuf, InvalidUriRef<IriBuf>> {
		UriRefBuf::new(self.into_bytes()).map_err(|InvalidUriRef(bytes)| unsafe {
			InvalidUriRef(Self::new_unchecked(String::from_utf8_unchecked(bytes)))
		})
	}
}

#[cfg(feature = "std")]
impl TryFrom<IriBuf> for UriBuf {
	type Error = InvalidUri<IriBuf>;

	fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
		value.try_into_uri()
	}
}

#[cfg(feature = "std")]
impl TryFrom<IriBuf> for UriRefBuf {
	type Error = InvalidUriRef<IriBuf>;

	fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
		value.try_into_uri_ref()
	}
}

impl IriRef {
	/// Converts this IRI reference into an URI, if possible.
	pub fn as_uri(&self) -> Option<&Uri> {
		Uri::new(self.as_bytes()).ok()
	}

	/// Converts this IRI reference into an URI reference, if possible.
	pub fn as_uri_ref(&self) -> Option<&UriRef> {
		UriRef::new(self.as_bytes()).ok()
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

#[cfg(feature = "std")]
impl IriRefBuf {
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
}
