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

impl TryFrom<&Iri> for url::Url {
	type Error = url::ParseError;

	fn try_from(iri: &Iri) -> Result<Self, Self::Error> {
		url::Url::parse(iri.as_str())
	}
}

impl TryFrom<IriBuf> for url::Url {
	type Error = url::ParseError;

	fn try_from(iri: IriBuf) -> Result<Self, Self::Error> {
		url::Url::parse(iri.as_str())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn url_to_uri() {
		let url = url::Url::parse("http://example.org/path?q=1#frag").unwrap();
		let uri: UriBuf = url.into();
		assert_eq!(uri.as_str(), "http://example.org/path?q=1#frag");
		assert_eq!(uri.scheme().as_str(), "http");
		assert_eq!(uri.authority().unwrap().host().as_str(), "example.org");
		assert_eq!(uri.path().as_str(), "/path");
		assert_eq!(uri.query().unwrap().as_str(), "q=1");
		assert_eq!(uri.fragment().unwrap().as_str(), "frag");
	}

	#[test]
	fn url_to_all_types() {
		let url = url::Url::parse("http://example.org").unwrap();
		let _: UriBuf = url.clone().into();
		let _: UriRefBuf = url.clone().into();
		let _: IriBuf = url.clone().into();
		let _: IriRefBuf = url.into();
	}

	#[test]
	fn uri_to_url() {
		let uri = Uri::new("http://example.org/path?q=1#frag").unwrap();
		let url: url::Url = uri.into();
		assert_eq!(url.as_str(), "http://example.org/path?q=1#frag");
	}

	#[test]
	fn uri_buf_to_url() {
		let uri = UriBuf::new("http://example.org/path".to_string()).unwrap();
		let url: url::Url = uri.into();
		assert_eq!(url.as_str(), "http://example.org/path");
	}

	#[test]
	fn iri_to_url() {
		let iri = Iri::new("http://example.org/path").unwrap();
		let url: url::Url = url::Url::try_from(iri).unwrap();
		assert_eq!(url.as_str(), "http://example.org/path");
	}

	#[test]
	fn iri_buf_to_url() {
		let iri = IriBuf::new("http://example.org/path".to_string()).unwrap();
		let url: url::Url = url::Url::try_from(iri).unwrap();
		assert_eq!(url.as_str(), "http://example.org/path");
	}

	#[test]
	fn round_trip() {
		let vectors = [
			"http://example.org/path?q=1#frag",
			"https://user@host:8080/a/b/c",
			"data:text/plain;base64,SGVsbG8=",
			"http://example.org",
			"http://example.org/",
			"http://example.org/?",
			"http://example.org/#",
		];

		for input in vectors {
			let url = url::Url::parse(input).unwrap();
			let uri: UriBuf = url.clone().into();
			let url2: url::Url = (&*uri).into();
			assert_eq!(url, url2, "round-trip failed for {input}");
		}
	}
}
