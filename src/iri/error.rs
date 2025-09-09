use std::borrow::Cow;

use super::{
	InvalidAuthority, InvalidFragment, InvalidHost, InvalidIri, InvalidIriRef, InvalidPath,
	InvalidPort, InvalidQuery, InvalidScheme, InvalidSegment, InvalidUserInfo,
};

macro_rules! iri_error {
	($($(#[$meta:meta])* $variant:ident : $ident:ident),*) => {
		#[derive(Debug, thiserror::Error)]
		pub enum IriError<T> {
			$(
				$(#[$meta])*
				$variant(#[from] $ident<T>)
			),*
		}

		$(
			impl<'a> From<$ident<String>> for IriError<Cow<'a, str>> {
				fn from($ident(value): $ident<String>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a str>> for IriError<Cow<'a, str>> {
				fn from($ident(value): $ident<&'a str>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}

			impl<'a> From<$ident<Vec<u8>>> for IriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<Vec<u8>>) -> Self {
					Self::$variant($ident(Cow::Owned(value)))
				}
			}

			impl<'a> From<$ident<&'a [u8]>> for IriError<Cow<'a, [u8]>> {
				fn from($ident(value): $ident<&'a [u8]>) -> Self {
					Self::$variant($ident(Cow::Borrowed(value)))
				}
			}
		)*
	};
}

iri_error! {
	#[error("invalid IRI: {0}")]
	Iri: InvalidIri,

	#[error("invalid IRI reference: {0}")]
	Reference: InvalidIriRef,

	#[error("invalid IRI scheme: {0}")]
	Scheme: InvalidScheme,

	#[error("invalid IRI authority: {0}")]
	Authority: InvalidAuthority,

	#[error("invalid IRI authority user info: {0}")]
	UserInfo: InvalidUserInfo,

	#[error("invalid IRI authority host: {0}")]
	Host: InvalidHost,

	#[error("invalid IRI authority port: {0}")]
	Port: InvalidPort,

	#[error("invalid IRI path: {0}")]
	Path: InvalidPath,

	#[error("invalid IRI path segment: {0}")]
	PathSegment: InvalidSegment,

	#[error("invalid IRI query: {0}")]
	Query: InvalidQuery,

	#[error("invalid IRI fragment: {0}")]
	Fragment: InvalidFragment
}
