//! This crates gives an implementation of
//! [Internationalized Resource Identifiers (IRIs)](https://en.wikipedia.org/wiki/Internationalized_resource_identifier) and IRI references following
//! [RFC 3987](https://tools.ietf.org/html/rfc3987) and
//! [RFC 3986](https://tools.ietf.org/html/rfc3986) defined by the
//! [Internet Engineering Task Force (IETF)](ietf.org).
//! IRIs are a superclass of
//! [Uniform Resource Identifier (URIs)](https://en.wikipedia.org/wiki/Uniform_resource_identifier) and
//! [Uniform Resource Locator (URLs)](https://en.wikipedia.org/wiki/Uniform_Resource_Locator)
//! used to uniquely identify objects across the web.
//! An IRI is defined as a sequence of characters with distinguishable components:
//! a scheme, an authority, a path, a query and a fragment.
//!
//! ```text
//!     foo://example.com:8042/over/there?name=ferret#nose
//!     \_/   \______________/\_________/ \_________/ \__/
//!      |           |            |            |        |
//!   scheme     authority       path        query   fragment
//! ```
//!
//! This crate provides the four types `Iri`, `IriBuf`, `IriRef` and `IriRefBuf`
//! to manipulate byte/string slices and buffers as IRIs and IRI references.
//! Theses allows the easy access and manipulation of every components.
//! It features:
//!   - borrowed and owned IRIs and IRI-reference;
//!   - mutable IRI buffers (in-place);
//!   - path normalization;
//!   - comparison modulo normalization;
//!   - IRI-reference resolution;
//!   - static IRI parsing with the [`static-iref`] crate and its `iri` macro; and
//!   - `serde` support (by enabling the `serde` feature).
//!
//! [`static-iref`]: https://crates.io/crates/static-iref
//!
//! ## Basic usage
//!
//! You can parse IRI strings by wrapping an `Iri` instance around a `str` slice.
//! Note that no memory allocation occurs using `Iri`, it only borrows the input data.
//! Access to each component is done in constant time.
//!
//! ```rust
//! use iref::Iri;
//!
//! # fn main() -> Result<(), iref::IriError<&'static str>> {
//! let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#frag")?;
//!
//! println!("scheme: {}", iri.scheme());
//! println!("authority: {}", iri.authority().unwrap());
//! println!("path: {}", iri.path());
//! println!("query: {}", iri.query().unwrap());
//! println!("fragment: {}", iri.fragment().unwrap());
//! #
//! # Ok(())
//! # }
//! ```
//!
//! IRIs can be created and modified using the `IriBuf` type.
//! With this type, the IRI is held in a single buffer,
//! modified in-place to reduce memory allocation and optimize memory accesses.
//! This also allows the conversion from `IriBuf` into `Iri`.
//!
//! ```rust
//! use iref::IriBuf;
//!
//! # fn main() -> Result<(), iref::IriError<std::borrow::Cow<'static, str>>> {
//! let mut iri = IriBuf::new("https://www.rust-lang.org".to_string())?;
//!
//! iri.authority_mut().unwrap().set_port(Some("40".try_into()?));
//! iri.set_path("/foo".try_into()?);
//! iri.path_mut().push("bar".try_into()?);
//! iri.set_query(Some("query".try_into()?));
//! iri.set_fragment(Some("fragment".try_into()?));
//!
//! assert_eq!(iri, "https://www.rust-lang.org:40/foo/bar?query#fragment");
//! # Ok(())
//! # }
//! ```
//!
//! The `try_into` method is used to ensure that each string is syntactically correct with regard to its corresponding component (for instance, it is not possible to replace `"query"` with `"query?"` since `?` is not a valid query character).
//!
//! ## Detailed Usage
//!
//! ### Path manipulation
//!
//! The IRI path is accessed through the `path` or `path_mut` methods.
//! It is possible to access the segments of a path using the iterator returned by the `segments` method.
//!
//! ```rust
//! # use iref::Iri;
//! # fn main() -> Result<(), iref::IriError<&'static str>> {
//! # let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#frag")?;
//! for segment in iri.path().segments() {
//! 	println!("{}", segment);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! One can use the `normalized_segments` method to iterate over the normalized
//! version of the path where dot segments (`.` and `..`) are removed.
//! In addition, it is possible to push or pop segments to a path using the
//! corresponding methods:
//! ```rust
//! # use iref::IriBuf;
//! # fn main() -> Result<(), iref::IriError<std::borrow::Cow<'static, str>>> {
//! let mut iri = IriBuf::new("https://rust-lang.org/a/c".to_string())?;
//! let mut path = iri.path_mut();
//!
//! path.pop();
//! path.push("b".try_into()?);
//! path.push("c".try_into()?);
//! path.push("".try_into()?); // the empty segment is valid.
//!
//! assert_eq!(iri.path(), "/a/b/c/");
//! # Ok(())
//! # }
//! ```
//!
//! ### IRI references
//!
//! This crate provides the two types `IriRef` and `IriRefBuf` to represent
//! IRI references. An IRI reference is either an IRI or a relative IRI.
//! Contrarily to regular IRIs, relative IRI references may have no scheme.
//!
//! ```rust
//! # use iref::{Iri, IriRef, IriRefBuf};
//! # fn main() -> Result<(), iref::IriError<&'static str>> {
//! let mut iri_ref = IriRefBuf::default(); // an IRI reference can be empty.
//!
//! // An IRI reference with a scheme is a valid IRI.
//! iri_ref.set_scheme(Some("https".try_into()?));
//! let iri: &Iri = iri_ref.as_iri().unwrap();
//!
//! // An IRI can be safely converted into an IRI reference.
//! let iri_ref: &IriRef = iri.into();
//! # Ok(())
//! # }
//! ```
//!
//! Given a base IRI, references can be resolved into a regular IRI using the
//! [Reference Resolution Algorithm](https://tools.ietf.org/html/rfc3986#section-5)
//! defined in [RFC 3986](https://tools.ietf.org/html/rfc3986).
//! This crate provides a *strict* implementation of this algorithm.
//!
//! ```rust
//! # use iref::{Iri, IriRef, IriRefBuf};
//! # fn main() -> Result<(), iref::IriError<::std::borrow::Cow<'static, str>>> {
//! let base_iri = Iri::new("http://a/b/c/d;p?q")?;
//! let mut iri_ref = IriRefBuf::new("g;x=1/../y".to_string())?;
//!
//! // non mutating resolution.
//! assert_eq!(iri_ref.resolved(base_iri), "http://a/b/c/y");
//!
//! // in-place resolution.
//! iri_ref.resolve(base_iri);
//! assert_eq!(iri_ref, "http://a/b/c/y");
//! # Ok(())
//! # }
//! ```
//!
//! This crate implements
//! [Errata 4547](https://www.rfc-editor.org/errata/eid4547) about the
//! abnormal use of dot segments in relative paths.
//! This means that for instance, the path `a/b/../../../` is normalized into
//! `../`.
//!
//! ### IRI comparison
//!
//! Here are the features of the IRI comparison method implemented in this crate.
//!
//! #### Protocol agnostic
//!
//! This implementation does not know anything about existing protocols.
//! For instance, even if the
//! [HTTP protocol](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol)
//! defines `80` as the default port,
//! the two IRIs `http://example.org` and `http://example.org:80` are **not** equivalent.
//!
//! #### Every `/` counts
//!
//! The path `/foo/bar` is **not** equivalent to `/foo/bar/`.
//!
//! #### Path normalization
//!
//! Paths are normalized during comparison by removing dot segments (`.` and `..`).
//! This means for instance that the paths `a/b/c` and `a/../a/./b/../b/c` **are**
//! equivalent.
//! Note however that this crate implements
//! [Errata 4547](https://www.rfc-editor.org/errata/eid4547) about the
//! abnormal use of dot segments in relative paths.
//! This means that for instance, the IRI `http:a/b/../../../` is equivalent to
//! `http:../` and **not** `http:`.
//!
//! #### Percent-encoded characters
//!
//! Thanks to the [`pct-str` crate](https://crates.io/crates/pct-str),
//! percent encoded characters are correctly handled.
//! The two IRIs `http://example.org` and `http://exa%6dple.org` **are** equivalent.
pub(crate) mod common;
pub mod iri;
pub mod uri;
pub(crate) mod utils;

pub use iri::{Iri, IriBuf, IriRef, IriRefBuf, InvalidIri, IriError};
pub use uri::{Uri, UriBuf, UriRef, UriRefBuf, InvalidUri, UriError};