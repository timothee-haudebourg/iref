mod buffer;

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
	/// Create a new IRI-reference slice from a bytes slice.
	///
	/// This may fail if the source slice is not UTF-8 encoded, or if is not a valid IRI-reference.
	#[inline]
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<IriRef<'a>, Error> {
		Ok(IriRef {
			data: buffer.as_ref(),
			p: ParsedIriRef::new(buffer)?,
		})
	}

	/// Get the underlying parsing data.
	#[inline]
	pub fn parsing_data(&self) -> ParsedIriRef {
		self.p
	}

	/// Build an IRI reference from a slice and parsing data.
	///
	/// This is unsafe since the input slice is not checked against the given parsing data.
	#[inline]
	pub const unsafe fn from_raw(data: &'a [u8], p: ParsedIriRef) -> IriRef<'a> {
		IriRef { p, data }
	}

	/// Get the length is the IRI-reference, in bytes.
	#[inline]
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Get a reference to the underlying bytes slice representing the IRI-reference.
	#[inline]
	pub fn as_ref(&self) -> &[u8] {
		self.data
	}

	/// Convert the IRI-refrence into its underlying bytes slice.
	#[inline]
	pub fn into_ref(self) -> &'a [u8] {
		self.data
	}

	/// Get the IRI-reference as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.data) }
	}

	/// Convert the IRI-reference into a string slice.
	#[inline]
	pub fn into_str(self) -> &'a str {
		unsafe { std::str::from_utf8_unchecked(self.data) }
	}

	/// Get the IRI-reference as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	/// Convert the IRI-reference into a percent-encoded string slice.
	#[inline]
	pub fn into_pct_str(self) -> &'a PctStr {
		unsafe { PctStr::new_unchecked(self.into_str()) }
	}

	/// Get the scheme of the IRI-reference.
	///
	/// The scheme is located at the very begining of the IRI-reference and delimited by an ending
	/// `:`.
	///
	/// # Example
	///
	/// ```
	/// # use iref::IriRef;
	/// assert_eq!(IriRef::new("foo://example.com:8042").unwrap().scheme().unwrap(), "foo");
	/// assert_eq!(IriRef::new("//example.com:8042").unwrap().scheme(), None);
	/// ```
	#[inline]
	pub fn scheme(&self) -> Option<Scheme> {
		if let Some(scheme_len) = self.p.scheme_len {
			Some(Scheme {
				data: &self.data[0..scheme_len],
			})
		} else {
			None
		}
	}

	/// Get the authority of the IRI-reference.
	///
	/// The authority is delimited by the `//` string, after the scheme.
	///
	/// # Example
	///
	/// ```
	/// # use iref::IriRef;
	/// assert_eq!(IriRef::new("foo://example.com:8042").unwrap().authority().unwrap().host(), "example.com");
	/// assert_eq!(IriRef::new("foo:").unwrap().authority(), None);
	/// ```
	#[inline]
	pub fn authority(&self) -> Option<Authority> {
		if let Some(authority) = self.p.authority {
			let offset = self.p.authority_offset();
			Some(Authority {
				data: &self.data[offset..(offset + authority.len())],
				p: authority,
			})
		} else {
			None
		}
	}

	/// Get the path of the IRI-reference.
	///
	/// The path is located just after the authority. It is always defined, even if empty.
	///
	/// # Example
	///
	/// ```
	/// # use iref::IriRef;
	/// assert_eq!(IriRef::new("foo:/a/b/c?query").unwrap().path(), "/a/b/c");
	/// assert!(IriRef::new("foo:#fragment").unwrap().path().is_empty());
	/// ```
	#[inline]
	pub fn path(&'a self) -> Path<'a> {
		let offset = self.p.path_offset();
		Path {
			data: &self.data[offset..(offset + self.p.path_len)],
		}
	}

	/// Get the query of the IRI-reference.
	///
	/// The query part is delimited by the `?` character after the path.
	///
	/// # Example
	///
	/// ```
	/// # use iref::IriRef;
	/// assert_eq!(IriRef::new("//example.org?query").unwrap().query().unwrap(), "query");
	/// assert!(IriRef::new("//example.org/foo/bar#fragment").unwrap().query().is_none());
	/// ```
	#[inline]
	pub fn query(&self) -> Option<Query> {
		if let Some(len) = self.p.query_len {
			let offset = self.p.query_offset();
			Some(Query {
				data: &self.data[offset..(offset + len)],
			})
		} else {
			None
		}
	}

	/// Get the fragment of the IRI-reference.
	///
	/// The fragment part is delimited by the `#` character after the query.
	///
	/// # Example
	///
	/// ```
	/// # use iref::IriRef;
	/// assert_eq!(IriRef::new("//example.org#foo").unwrap().fragment().unwrap(), "foo");
	/// assert!(IriRef::new("//example.org").unwrap().fragment().is_none());
	/// ```
	#[inline]
	pub fn fragment(&self) -> Option<Fragment> {
		if let Some(len) = self.p.fragment_len {
			let offset = self.p.fragment_offset();
			Some(Fragment {
				data: &self.data[offset..(offset + len)],
			})
		} else {
			None
		}
	}

	/// Convert the IRI-reference into an IRI, if possible.
	///
	/// An IRI-reference is a valid IRI only if it has a defined [`Scheme`].
	#[inline]
	pub fn into_iri(self) -> Result<Iri<'a>, IriRef<'a>> {
		self.try_into()
	}

	/// Resolve the IRI reference against the given *base IRI*.
	///
	/// Return the resolved IRI.
	/// See the [`IriRefBuf::resolve`] method for more informations about the resolution process.
	#[inline]
	pub fn resolved<'b, Base: Into<Iri<'b>>>(&self, base_iri: Base) -> IriBuf {
		let mut iri_ref: IriRefBuf = self.into();
		iri_ref.resolve(base_iri);
		iri_ref.try_into().unwrap()
	}

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
			match self.path().suffix(prefix.path()) {
				Some(suffix_path) => Some((suffix_path, self.query(), self.fragment())),
				None => None,
			}
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

		while let Some(segment) = base_segments.next() {
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

impl<'a> AsIriRef for IriRef<'a> {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		*self
	}
}

impl<'a> fmt::Display for IriRef<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for IriRef<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for IriRef<'a> {
	#[inline]
	fn eq(&self, other: &IriRef) -> bool {
		self.scheme() == other.scheme()
			&& self.fragment() == other.fragment()
			&& self.authority() == other.authority()
			&& self.path() == other.path()
			&& self.query() == other.query()
	}
}

impl<'a> Eq for IriRef<'a> {}

impl<'a> cmp::PartialEq<IriRefBuf> for IriRef<'a> {
	#[inline]
	fn eq(&self, other: &IriRefBuf) -> bool {
		*self == other.as_iri_ref()
	}
}

impl<'a> cmp::PartialEq<Iri<'a>> for IriRef<'a> {
	#[inline]
	fn eq(&self, other: &Iri<'a>) -> bool {
		*self == other.as_iri_ref()
	}
}

impl<'a> cmp::PartialEq<IriBuf> for IriRef<'a> {
	#[inline]
	fn eq(&self, other: &IriBuf) -> bool {
		*self == other.as_iri_ref()
	}
}

impl<'a> cmp::PartialEq<&'a str> for IriRef<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = IriRef::new(other) {
			self == &other
		} else {
			false
		}
	}
}

impl<'a> PartialOrd for IriRef<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for IriRef<'a> {
	#[inline]
	fn cmp(&self, other: &IriRef<'a>) -> Ordering {
		if self.scheme() == other.scheme() {
			if self.authority() == other.authority() {
				if self.path() == other.path() {
					if self.query() == other.query() {
						self.fragment().cmp(&other.fragment())
					} else {
						self.query().cmp(&other.query())
					}
				} else {
					self.path().cmp(&other.path())
				}
			} else {
				self.authority().cmp(&other.authority())
			}
		} else {
			self.scheme().cmp(&other.scheme())
		}
	}
}

impl<'a> PartialOrd<IriRefBuf> for IriRef<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<Iri<'a>> for IriRef<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriBuf> for IriRef<'a> {
	#[inline]
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<&'a IriRefBuf> for IriRef<'a> {
	#[inline]
	fn from(iri_ref_buf: &'a IriRefBuf) -> IriRef<'a> {
		iri_ref_buf.as_iri_ref()
	}
}

impl<'a> From<Iri<'a>> for IriRef<'a> {
	#[inline]
	fn from(iri: Iri<'a>) -> IriRef<'a> {
		iri.as_iri_ref()
	}
}

impl<'a> From<&'a IriBuf> for IriRef<'a> {
	#[inline]
	fn from(iri_ref_buf: &'a IriBuf) -> IriRef<'a> {
		iri_ref_buf.as_iri_ref()
	}
}

impl<'a> From<Path<'a>> for IriRef<'a> {
	#[inline]
	fn from(path: Path<'a>) -> IriRef<'a> {
		path.into_iri_ref()
	}
}

impl<'a> From<&'a PathBuf> for IriRef<'a> {
	#[inline]
	fn from(path: &'a PathBuf) -> IriRef<'a> {
		path.as_path().into()
	}
}

impl<'a> Hash for IriRef<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.scheme().hash(hasher);
		self.authority().hash(hasher);
		self.path().hash(hasher);
		self.query().hash(hasher);
		self.fragment().hash(hasher);
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
}
