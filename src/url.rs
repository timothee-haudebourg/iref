//! Compatibility layer with the `url` crate.
use crate::{Iri, IriBuf, IriRefBuf, Uri, UriBuf, UriRefBuf};

impl From<url::Url> for UriBuf {
	fn from(url: url::Url) -> Self {
		unsafe {
			// SAFETY: An `Url` is a valid URI.
			Self::new_unchecked(String::from(url))
		}
	}
}

impl From<url::Url> for UriRefBuf {
	fn from(url: url::Url) -> Self {
		UriBuf::from(url).into()
	}
}

impl From<url::Url> for IriBuf {
	fn from(url: url::Url) -> Self {
		UriBuf::from(url).into_iri()
	}
}

impl From<url::Url> for IriRefBuf {
	fn from(url: url::Url) -> Self {
		UriBuf::from(url).into_iri_ref()
	}
}

impl From<&Uri> for url::Url {
	fn from(uri: &Uri) -> Self {
		url::Url::parse(uri.as_str()).unwrap()
	}
}

impl From<UriBuf> for url::Url {
	fn from(uri: UriBuf) -> Self {
		url::Url::parse(uri.as_str()).unwrap()
	}
}

impl From<&Iri> for url::Url {
	fn from(iri: &Iri) -> Self {
		url::Url::parse(iri.as_str()).unwrap()
	}
}

impl From<IriBuf> for url::Url {
	fn from(iri: IriBuf) -> Self {
		url::Url::parse(iri.as_str()).unwrap()
	}
}
