use super::{
	InvalidAuthority, InvalidFragment, InvalidHost, InvalidPath, InvalidPort, InvalidQuery,
	InvalidScheme, InvalidSegment, InvalidUri, InvalidUriRef, InvalidUserInfo,
};

macro_rules! uri_error {
	($($(#[$meta:meta])* $variant:ident : $ident:ident),*) => {
		#[derive(Debug, thiserror::Error)]
		pub enum UriError<T> {
			$(
				$(#[$meta])*
				$variant(#[from] $ident<T>)
			),*
		}

		$(
		    #[cfg(feature = "std")]
			impl<'a> From<$ident<String>> for UriError<std::borrow::Cow<'a, str>> {
				fn from($ident(value): $ident<String>) -> Self {
					Self::$variant($ident(std::borrow::Cow::Owned(value)))
				}
			}

			#[cfg(feature = "std")]
			impl<'a> From<$ident<&'a str>> for UriError<std::borrow::Cow<'a, str>> {
				fn from($ident(value): $ident<&'a str>) -> Self {
					Self::$variant($ident(std::borrow::Cow::Borrowed(value)))
				}
			}

			#[cfg(feature = "std")]
			impl<'a> From<$ident<Vec<u8>>> for UriError<std::borrow::Cow<'a, [u8]>> {
				fn from($ident(value): $ident<Vec<u8>>) -> Self {
					Self::$variant($ident(std::borrow::Cow::Owned(value)))
				}
			}

			#[cfg(feature = "std")]
			impl<'a> From<$ident<&'a [u8]>> for UriError<std::borrow::Cow<'a, [u8]>> {
				fn from($ident(value): $ident<&'a [u8]>) -> Self {
					Self::$variant($ident(std::borrow::Cow::Borrowed(value)))
				}
			}
		)*
	};
}

uri_error! {
	#[error("invalid URI: {0}")]
	Uri: InvalidUri,

	#[error("invalid URI reference: {0}")]
	Reference: InvalidUriRef,

	#[error("invalid URI scheme: {0}")]
	Scheme: InvalidScheme,

	#[error("invalid URI authority: {0}")]
	Authority: InvalidAuthority,

	#[error("invalid URI authority user info: {0}")]
	UserInfo: InvalidUserInfo,

	#[error("invalid URI authority host: {0}")]
	Host: InvalidHost,

	#[error("invalid URI authority port: {0}")]
	Port: InvalidPort,

	#[error("invalid URI path: {0}")]
	Path: InvalidPath,

	#[error("invalid URI path segment: {0}")]
	PathSegment: InvalidSegment,

	#[error("invalid URI query: {0}")]
	Query: InvalidQuery,

	#[error("invalid URI fragment: {0}")]
	Fragment: InvalidFragment
}
