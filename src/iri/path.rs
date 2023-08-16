use static_regular_grammar::RegularGrammar;

mod segment;

pub use segment::*;

use crate::common::path::{NormalizedSegmentsImpl, PathBufImpl, PathImpl, SegmentsImpl};

use super::PathMut;

/// IRI path.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "ipath",
	cache = "automata/iri/path.aut.cbor"
)]
#[grammar(sized(PathBuf, derive(Debug, Display)))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Path(str);

impl PathImpl for Path {
	const EMPTY: &'static Self = Self::EMPTY;

	const EMPTY_ABSOLUTE: &'static Self = Self::EMPTY_ABSOLUTE;

	type Segment = Segment;

	type Owned = PathBuf;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(std::str::from_utf8_unchecked(bytes))
	}

	#[inline(always)]
	fn as_bytes(&self) -> &[u8] {
		self.as_bytes()
	}

	fn to_path_buf(&self) -> Self::Owned {
		unsafe { PathBuf::new_unchecked(self.to_string()) }
	}
}

impl PathBufImpl for PathBuf {
	type Borrowed = Path;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}
}

impl Path {
	/// The empty absolute path `/`.
	pub const EMPTY_ABSOLUTE: &'static Self = unsafe { Self::new_unchecked("/") };

	/// Returns the number of segments in the path.
	///
	/// This computes in linear time w.r.t the number of segments. It is
	/// equivalent to `path.segments().count()`.
	#[inline]
	pub fn segment_count(&self) -> usize {
		self.segments().count()
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path has no segments.
	/// The absolute path `/` is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		PathImpl::is_empty(self)
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	#[inline]
	pub fn is_absolute(&self) -> bool {
		PathImpl::is_absolute(self)
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	#[inline]
	pub fn is_relative(&self) -> bool {
		PathImpl::is_relative(self)
	}

	pub fn first(&self) -> Option<&Segment> {
		PathImpl::first(self)
	}

	pub fn last(&self) -> Option<&Segment> {
		PathImpl::last(self)
	}

	/// Produces an iterator over the segments of the IRI path.
	///
	/// Empty segments are preserved: the path `a//b` will raise the three
	/// segments `a`, `` and `b`. The absolute path `/` has no segments, but
	/// the path `/a/` has two segments, `a` and ``.
	///
	/// No normalization occurs with `.` and `..` segments. See the
	/// [`Self::normalized_segments`] to iterate over the normalized segments
	/// of a path.
	#[inline]
	pub fn segments(&self) -> Segments {
		Segments(PathImpl::segments(self))
	}

	/// Iterate over the normalized segments of the path.
	///
	/// Remove the special dot segments `..` and `.` from the iteration using
	/// the usual path semantics for dot segments. This may be expensive for
	/// large paths since it will need to internally normalize the path first.
	#[inline]
	pub fn normalized_segments(&self) -> NormalizedSegments {
		NormalizedSegments(PathImpl::normalized_segments(self))
	}

	#[inline]
	pub fn normalized(&self) -> PathBuf {
		PathImpl::normalized(self)
	}

	/// Returns the last segment of the path, if there is one, unless it is
	/// empty.
	///
	/// This does not consider the normalized version of the path, dot segments
	/// are preserved.
	#[inline]
	pub fn file_name(&self) -> Option<&Segment> {
		PathImpl::file_name(self)
	}

	/// Returns the path without its final segment, if there is one.
	///
	/// ```
	/// # use iref::iri::Path;
	/// assert_eq!(Path::new("/foo/bar").unwrap().parent().unwrap(), "/foo");
	/// assert_eq!(Path::new("/foo").unwrap().parent().unwrap(), "/");
	/// assert_eq!(Path::new("//foo").unwrap().parent().unwrap(), "/./");
	/// assert_eq!(Path::new("/").unwrap().parent(), None);
	/// ```
	#[inline]
	pub fn parent(&self) -> Option<&Self> {
		PathImpl::parent(self)
	}

	/// Returns the path without its final segment, if there is one.
	///
	/// ```
	/// # use iref::iri::Path;
	/// assert_eq!(Path::new("/foo/bar").unwrap().parent_or_empty(), "/foo");
	/// assert_eq!(Path::new("/foo").unwrap().parent_or_empty(), "/");
	/// assert_eq!(Path::new("//foo").unwrap().parent_or_empty(), "/./");
	/// assert_eq!(Path::new("/").unwrap().parent_or_empty(), "/");
	/// assert_eq!(Path::new("").unwrap().parent_or_empty(), "");
	/// ```
	#[inline]
	pub fn parent_or_empty(&self) -> &Self {
		PathImpl::parent_or_empty(self)
	}

	/// Get the suffix part of this path, if any, with regard to the given prefix path.
	///
	/// Returns `Some(suffix)` if this path is of the form `prefix/suffix` where `prefix` is given
	/// as parameter. Returns `None` otherwise.
	///
	/// Both paths are normalized during the process.
	/// The result is a normalized suffix path.
	///
	/// # Example
	/// ```
	/// # use std::convert::TryFrom;
	/// use iref::iri::{Path, PathBuf};
	///
	/// let prefix = Path::new("/foo/bar").unwrap();
	/// let path = Path::new("/foo/bar/baz").unwrap();
	/// let suffix: PathBuf = path.suffix(prefix).unwrap();
	///
	/// assert_eq!(suffix.as_str(), "baz");
	/// ```
	#[inline]
	pub fn suffix(&self, prefix: &Self) -> Option<PathBuf> {
		PathImpl::suffix(self, prefix)
	}
}

impl<'a> IntoIterator for &'a Path {
	type Item = &'a Segment;
	type IntoIter = Segments<'a>;

	#[inline]
	fn into_iter(self) -> Segments<'a> {
		self.segments()
	}
}

impl PartialEq for Path {
	#[inline]
	fn eq(&self, other: &Path) -> bool {
		if self.is_absolute() == other.is_absolute() {
			let self_segments = self.normalized_segments();
			let other_segments = other.normalized_segments();
			self_segments.len() == other_segments.len()
				&& self_segments.zip(other_segments).all(|(a, b)| a == b)
		} else {
			false
		}
	}
}

impl PartialEq<str> for Path {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl<'a> PartialEq<&'a str> for Path {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<String> for Path {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for Path {}

impl PartialOrd for Path {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Path {
	#[inline]
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		use std::cmp::Ordering;
		if self.is_absolute() == other.is_absolute() {
			let mut self_segments = self.normalized_segments();
			let mut other_segments = other.normalized_segments();

			loop {
				match (self_segments.next(), other_segments.next()) {
					(None, None) => return Ordering::Equal,
					(Some(_), None) => return Ordering::Greater,
					(None, Some(_)) => return Ordering::Less,
					(Some(a), Some(b)) => match a.cmp(b) {
						Ordering::Greater => return Ordering::Greater,
						Ordering::Less => return Ordering::Less,
						Ordering::Equal => (),
					},
				}
			}
		} else if self.is_absolute() {
			Ordering::Greater
		} else {
			Ordering::Less
		}
	}
}

impl std::hash::Hash for Path {
	#[inline]
	fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
		self.is_absolute().hash(hasher);
		self.normalized_segments().for_each(move |s| s.hash(hasher))
	}
}

pub struct Segments<'a>(SegmentsImpl<'a, Path>);

impl<'a> Iterator for Segments<'a> {
	type Item = &'a Segment;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl<'a> DoubleEndedIterator for Segments<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

pub struct NormalizedSegments<'a>(NormalizedSegmentsImpl<'a, Path>);

impl<'a> Iterator for NormalizedSegments<'a> {
	type Item = &'a Segment;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<&'a Segment> {
		self.0.next()
	}
}

impl<'a> DoubleEndedIterator for NormalizedSegments<'a> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<'a> ExactSizeIterator for NormalizedSegments<'a> {}

impl PathBuf {
	/// Returns a mutable reference to the interior bytes.
	///
	/// # Safety
	///
	/// This function is unsafe because the returned `&mut Vec` allows writing
	/// bytes which are not valid in a path. If this constraint is violated,
	/// using the original `PathBuf` after dropping the `&mut Vec` may violate
	/// memory safety, as the rest of the library assumes that `PathBuf` are
	/// valid paths.
	pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		self.0.as_mut_vec()
	}

	pub fn as_path_mut(&mut self) -> PathMut {
		PathMut::from_path(self)
	}

	pub fn push(&mut self, segment: &Segment) {
		self.as_path_mut().push(segment)
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead.
	pub fn pop(&mut self) {
		self.as_path_mut().pop()
	}

	pub fn clear(&mut self) {
		self.as_path_mut().clear()
	}

	/// Push the given segment to this path using the `.` and `..` segments semantics.
	#[inline]
	pub fn symbolic_push(&mut self, segment: &Segment) {
		self.as_path_mut().symbolic_push(segment)
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	#[inline]
	pub fn symbolic_append<'s, P: IntoIterator<Item = &'s Segment>>(&mut self, path: P) {
		self.as_path_mut().symbolic_append(path)
	}

	#[inline]
	pub fn normalize(&mut self) {
		self.as_path_mut().normalize()
	}
}

impl PartialEq<str> for PathBuf {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl<'a> PartialEq<&'a str> for PathBuf {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<String> for PathBuf {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other.as_str()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let path = Path::EMPTY;
		assert!(path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn empty_absolute() {
		let path = Path::EMPTY_ABSOLUTE;
		assert!(path.is_empty());
		assert!(path.is_absolute());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn non_empty() {
		let path = Path::new("a/b").unwrap();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "a");
		assert!(segments.next().unwrap().as_str() == "b");
		assert!(segments.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let path = Path::new("/foo/bar").unwrap();
		assert!(!path.is_empty());
		assert!(path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "foo");
		assert!(segments.next().unwrap().as_str() == "bar");
		assert!(segments.next().is_none());
	}

	#[test]
	fn next_segment() {
		let vectors = [
			("foo/bar", 0, Some(("foo", 4))),
			("foo/bar", 4, Some(("bar", 8))),
			("foo/bar", 8, None),
			("foo/bar/", 8, Some(("", 9))),
			("foo/bar/", 9, None),
			("//foo", 1, Some(("", 2))),
		];

		for (input, offset, expected) in vectors {
			unsafe {
				assert_eq!(
					Path::new(input).unwrap().next_segment_from(offset),
					expected.map(|(e, i)| (Segment::new(e).unwrap(), i))
				)
			}
		}
	}

	#[test]
	fn previous_segment() {
		let vectors = [
			("/foo/bar", 1, None),
			("foo/bar", 0, None),
			("foo/bar", 4, Some(("foo", 0))),
			("foo/bar", 8, Some(("bar", 4))),
			("foo/bar/", 8, Some(("bar", 4))),
			("foo/bar/", 9, Some(("", 8))),
			("//a/b", 4, Some(("a", 2))),
		];

		for (input, offset, expected) in vectors {
			unsafe {
				assert_eq!(
					Path::new(input).unwrap().previous_segment_from(offset),
					expected.map(|(e, i)| (Segment::new(e).unwrap(), i))
				)
			}
		}
	}

	#[test]
	fn first_segment() {
		let vectors = [
			("", None),
			("/", None),
			("//", Some("")),
			("/foo/bar", Some("foo")),
		];

		for (input, expected) in vectors {
			assert_eq!(
				Path::new(input).unwrap().first(),
				expected.map(|e| Segment::new(e).unwrap())
			)
		}
	}

	#[test]
	fn segments() {
		let vectors: [(&str, &[&str]); 8] = [
			("", &[]),
			("foo", &["foo"]),
			("/foo", &["foo"]),
			("foo/", &["foo", ""]),
			("/foo/", &["foo", ""]),
			("a/b/c/d", &["a", "b", "c", "d"]),
			("a/b//c/d", &["a", "b", "", "c", "d"]),
			("//a/b/foo//bar/", &["", "a", "b", "foo", "", "bar", ""]),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(segments
				.into_iter()
				.zip(expected)
				.all(|(a, b)| a.as_str() == *b))
		}
	}

	#[test]
	fn segments_rev() {
		let vectors: [(&str, &[&str]); 8] = [
			("", &[]),
			("foo", &["foo"]),
			("/foo", &["foo"]),
			("foo/", &["foo", ""]),
			("/foo/", &["foo", ""]),
			("a/b/c/d", &["a", "b", "c", "d"]),
			("a/b//c/d", &["a", "b", "", "c", "d"]),
			("//a/b/foo//bar/", &["", "a", "b", "foo", "", "bar", ""]),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().rev().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(segments
				.into_iter()
				.zip(expected.into_iter().rev())
				.all(|(a, b)| a.as_str() == *b))
		}
	}

	#[test]
	fn normalized() {
		let vectors = [
			("", ""),
			("a/b/c", "a/b/c"),
			("a/..", ""),
			("a/b/..", "a"),
			("a/b/../", "a/"),
			("a/b/c/..", "a/b"),
			("a/b/c/.", "a/b/c"),
			("a/../..", ".."),
			("/a/../..", "/.."),
		];

		for (input, expected) in vectors {
			eprintln!("{input}, {expected}");
			let path = Path::new(input).unwrap();
			let output = path.normalized();
			assert_eq!(output.as_str(), expected);
		}
	}

	#[test]
	fn eq() {
		let vectors = [
			("a/b/c", "a/b/c"),
			("a/b/c", "a/b/c/."),
			("a/b/c/", "a/b/c/./"),
			("a/b/c", "a/b/../b/c"),
			("a/b/c/..", "a/b"),
			("a/..", ""),
			("/a/..", "/"),
			("a/../..", ".."),
			("/a/../..", "/.."),
			("a/b/c/./", "a/b/c/"),
			("a/b/c/../", "a/b/"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_eq!(a, b)
		}
	}

	#[test]
	fn ne() {
		let vectors = [
			("a/b/c", "a/b/c/"),
			("a/b/c/", "a/b/c/."),
			("a/b/c/../", "a/b"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_ne!(a, b)
		}
	}

	#[test]
	fn file_name() {
		let vectors = [("//a/b/foo//bar/", None), ("//a/b/foo//bar", Some("bar"))];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.file_name().map(Segment::as_str), expected)
		}
	}

	#[test]
	fn parent() {
		let vectors = [
			("", None),
			("/", None),
			(".", None),
			("//a/b/foo//bar", Some("//a/b/foo/")),
			("//a/b/foo//", Some("//a/b/foo/")),
			("//a/b/foo/", Some("//a/b/foo")),
			("//a/b/foo", Some("//a/b")),
			("//a/b", Some("//a")),
			("//a", Some("/./")),
			("/./", Some("/.")),
			("/.", Some("/")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.parent().map(Path::as_str), expected)
		}
	}

	#[test]
	fn suffix() {
		let vectors = [
			("/foo/bar/baz", "/foo/bar", Some("baz")),
			("//foo", "/", Some(".//foo")),
			("/a/b/baz", "/foo/bar", None),
		];

		for (path, prefix, expected_suffix) in vectors {
			let path = Path::new(path).unwrap();
			let suffix = path.suffix(Path::new(prefix).unwrap());
			assert_eq!(suffix.as_deref().map(Path::as_str), expected_suffix)
		}
	}
}
