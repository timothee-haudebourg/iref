#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod common;
pub mod iri;
pub mod uri;
pub(crate) mod utils;

pub use iri::{InvalidIri, Iri, IriBuf, IriError, IriRef, IriRefBuf};
pub use uri::{InvalidUri, Uri, UriBuf, UriError, UriRef, UriRefBuf};

#[cfg(not(feature = "std"))]
pub(crate) use alloc::borrow::Cow;
#[cfg(feature = "std")]
pub(crate) use std::borrow::Cow;
