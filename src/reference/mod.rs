mod buffer;

use std::borrow::Borrow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::{cmp, fmt};
// use log::*;
use pct_str::PctStr;

use crate::parsing::ParsedIriRef;
use crate::{
	AsIriRef, Authority, Error, Fragment, Iri, IriBuf, Path, PathBuf, Query, Scheme, Segment,
};

pub use self::buffer::*;

/// IRI-reference slice.
///
/// Wrapper around a borrowed bytes slice representing an IRI-reference.
/// An IRI-reference can be seen as an [`Iri`] with an optional [`Scheme`].
/// IRI-references are resolved against a *base IRI* into a proper IRI using
/// the [Reference Resolution Algorithm](https://tools.ietf.org/html/rfc3986#section-5) provided
/// by the [`resolved`](`IriRef::resolved`) method.
///
/// ## Example
///
/// ```rust
/// # extern crate iref;
/// # use std::convert::TryInto;
/// # use iref::{Iri, IriRef, IriRefBuf};
/// # fn main() -> Result<(), iref::Error> {
/// let base_iri = Iri::new("http://a/b/c/d;p?q")?;
/// let mut iri_ref = IriRefBuf::new("g;x=1/../y")?;
///
/// assert_eq!(iri_ref.resolved(base_iri), "http://a/b/c/y");
/// # Ok(())
/// # }
#[derive(Clone, Copy)]
pub struct IriRef<'a> {
	pub(crate) p: ParsedIriRef,
	pub(crate) data: &'a [u8],
}

impl<'a> IriRef<'a> {
	/// Get the suffix of this IRI reference, if any, with regard to the given prefix IRI reference..
	///
	/// Returns `Some((suffix, query, fragment))` if this IRI reference is of the form
	/// `prefix/suffix?query#fragment` where `prefix` is given as parameter.
	/// Returns `None` otherwise.
	/// If the `suffix` scheme or authority is different from this path, it will return `None`.
	///
	/// See [`Path::suffix`] for more details.
	#[inline]
	pub fn suffix<'b, Prefix: Into<IriRef<'b>>>(
		&self,
		prefix: Prefix,
	) -> Option<(PathBuf, Option<Query>, Option<Fragment>)> {
		let prefix = prefix.into();
		if self.scheme() == prefix.scheme() && self.authority() == prefix.authority() {
			self.path()
				.suffix(prefix.path())
				.map(|suffix_path| (suffix_path, self.query(), self.fragment()))
		} else {
			None
		}
	}

	/// The IRI reference without the file name, query and fragment.
	///
	/// # Example
	/// ```
	/// # use iref::IriRef;
	/// let a = IriRef::new("https://crates.io/crates/iref?query#fragment").unwrap();
	/// let b = IriRef::new("https://crates.io/crates/iref/?query#fragment").unwrap();
	/// assert_eq!(a.base(), "https://crates.io/crates/");
	/// assert_eq!(b.base(), "https://crates.io/crates/iref/")
	/// ```
	#[inline]
	pub fn base(&self) -> IriRef {
		let directory_path = self.path().directory();

		let p = ParsedIriRef {
			path_len: directory_path.len(),
			query_len: None,
			fragment_len: None,
			..self.p
		};

		let len = p.len();

		IriRef {
			p,
			data: &self.data[0..len],
		}
	}

	/// Get this IRI reference relatively to the given one.
	///
	/// # Example
	/// ```
	/// # use iref::IriRef;
	/// let a = IriRef::new("https://crates.io/").unwrap();
	/// let b = IriRef::new("https://crates.io/crates/iref").unwrap();
	/// let c = IriRef::new("https://crates.io/crates/json-ld").unwrap();
	/// assert_eq!(b.relative_to(a), "crates/iref");
	/// assert_eq!(a.relative_to(b), "../");
	/// assert_eq!(b.relative_to(c), "iref");
	/// assert_eq!(c.relative_to(b), "json-ld");
	/// ```
	#[inline]
	pub fn relative_to<'b, Other: Into<IriRef<'b>>>(&self, other: Other) -> IriRefBuf {
		let other = other.into();
		let mut result = IriRefBuf::default();

		match (self.scheme(), other.scheme()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => return self.into(),
		}

		match (self.authority(), other.authority()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => return self.into(),
		}

		let mut self_segments = self.path().into_normalized_segments().peekable();
		let mut base_segments = other.path().into_normalized_segments().peekable();

		loop {
			match base_segments.peek() {
				Some(a) if a.is_open() || self.query().is_some() || self.fragment().is_some() => {
					match self_segments.peek() {
						Some(b) if a.as_pct_str() == b.as_pct_str() => {
							base_segments.next();
							self_segments.next();
						}
						_ => break,
					}
				}
				_ => break,
			}
		}

		for segment in base_segments {
			if segment.is_open() {
				result.path_mut().push(Segment::parent());
				result.path_mut().open();
			}
		}

		for segment in self_segments {
			result.path_mut().push(segment)
		}

		result.set_query(self.query());
		result.set_fragment(self.fragment());

		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn relative_to() {
		let base =
			IriRef::new("https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld").unwrap();
		let challenges = [
			(
				"https://w3c.github.io/json-ld-api/tests/compact/link",
				"link",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld#fragment-works",
				"#fragment-works",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld?query=works",
				"?query=works",
			),
			("https://w3c.github.io/json-ld-api/tests/", "../"),
			("https://w3c.github.io/json-ld-api/", "../../"),
			("https://w3c.github.io/json-ld-api/parent", "../../parent"),
			(
				"https://w3c.github.io/json-ld-api/parent#fragment",
				"../../parent#fragment",
			),
			(
				"https://w3c.github.io/parent-parent-eq-root",
				"../../../parent-parent-eq-root",
			),
			(
				"http://example.org/scheme-relative",
				"http://example.org/scheme-relative",
			),
			(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld",
				"0066-in.jsonld",
			),
		];

		for (input, expected) in &challenges {
			let input = IriRef::new(input).unwrap();
			assert_eq!(input.relative_to(base), *expected)
		}
	}

	// https://github.com/timothee-haudebourg/iref/issues/14
	#[test]
	fn reference_resolution_with_scheme_no_disambiguation() {
		let base = Iri::new("scheme:a:b/").unwrap();
		let mut iri = IriRefBuf::new("Foo").unwrap();
		iri.resolve(base);

		assert_eq!(iri.to_string(), "scheme:a:b/Foo")
	}
}
