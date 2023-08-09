use smallvec::SmallVec;
use static_regular_grammar::RegularGrammar;
use std::fmt;

mod segment;

pub use segment::*;

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

impl Path {
	/// The empty absolute path `/`.
	pub const EMPTY_ABSOLUTE: &'static Self = unsafe { Self::new_unchecked("/") };

	/// Returns the number of segments in the path.
	///
	/// This computes in linear time w.r.t the number of segments. It is
	/// equivalent to `path.segments().count()`.
	#[inline]
	pub fn len(&self) -> usize {
		self.segments().count()
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path has no segments.
	/// The absolute path `/` is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty() || self.0 == *"/"
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	#[inline]
	pub fn is_absolute(&self) -> bool {
		self.0.starts_with("/")
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	#[inline]
	pub fn is_relative(&self) -> bool {
		!self.is_absolute()
	}

	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() {
			1
		} else {
			0
		}
	}

	pub fn first(&self) -> Option<&Segment> {
		if self.is_empty() {
			None
		} else {
			Some(unsafe { self.segment_at(self.first_segment_offset()).0 })
		}
	}

	pub fn last(&self) -> Option<&Segment> {
		if self.is_empty() {
			None
		} else {
			unsafe {
				self.previous_segment_from(self.len() + 1)
					.map(|(segment, _)| segment)
			}
		}
	}

	/// Returns the segment starting at the given byte offset and the offset
	/// of the next segment, if any.
	///
	/// # Safety
	///
	/// A segment must start at the given offset.
	unsafe fn segment_at(&self, offset: usize) -> (&Segment, usize) {
		let mut i = offset;

		let bytes = self.as_bytes();
		while i < bytes.len() && !matches!(bytes[i], b'/' | b'?' | b'#') {
			i += 1
		}

		(Segment::new_unchecked(&self.0[offset..i]), i + 1)
	}

	/// Returns the segment following a previous segment ending at the given
	/// offset.
	///
	/// # Safety
	///
	/// - as the start of a segment; or
	/// - at `path.as_bytes().len() + 1`.
	unsafe fn next_segment_from(&self, offset: usize) -> Option<(&Segment, usize)> {
		let bytes = self.as_bytes();
		if offset <= bytes.len() {
			Some(self.segment_at(offset))
		} else {
			None
		}
	}

	/// Returns the segment preceding the segment starting at the given offset,
	/// and its offset.
	///
	/// # Safety
	///
	/// The offset must be either:
	/// - as the start of a segment; or
	/// - at `path.as_bytes().len() + 1`.
	unsafe fn previous_segment_from(&self, offset: usize) -> Option<(&Segment, usize)> {
		// //a/b
		if offset >= 2 {
			let first_offset = self.first_segment_offset();
			let bytes = self.as_bytes();
			// offset is at the end of a segment.
			let mut i = offset - 2;
			while i > first_offset && bytes[i] != b'/' {
				i -= 1
			}

			if bytes[i] == b'/' {
				let j = i + 1;
				Some((self.segment_at(j).0, j))
			} else {
				Some((self.segment_at(first_offset).0, first_offset))
			}
		} else {
			// offset is at the first segment.
			None
		}
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
		if self.is_empty() {
			Segments::Empty
		} else {
			Segments::NonEmpty {
				path: self,
				offset: self.first_segment_offset(),
				back_offset: self.as_bytes().len() + 1,
			}
		}
	}

	/// Iterate over the normalized segments of the path.
	///
	/// Remove the special dot segments `..` and `.` from the iteration using
	/// the usual path semantics for dot segments. This may be expensive for
	/// large paths since it will need to internally normalize the path first.
	#[inline]
	pub fn normalized_segments(&self) -> NormalizedSegments {
		NormalizedSegments::new(self)
	}

	#[inline]
	pub fn normalized(&self) -> PathBuf {
		let mut result: PathBuf = if self.is_absolute() {
			Path::EMPTY_ABSOLUTE.to_owned()
		} else {
			Path::EMPTY.to_owned()
		};

		for segment in self.segments() {
			result.as_path_mut().symbolic_push(segment)
		}

		result
	}

	/// Return the path directory part.
	///
	/// This correspond to the path without everything after the right most `/`.
	#[inline]
	pub fn directory(&self) -> &Self {
		if self.is_empty() {
			Self::EMPTY
		} else {
			let bytes = self.as_bytes();
			let mut last = bytes.len() - 1;

			loop {
				if bytes[last] == b'/' {
					break;
				}

				if last == 0 {
					return Self::EMPTY;
				}

				last -= 1;
			}

			unsafe { Self::new_unchecked(std::str::from_utf8_unchecked(&bytes[..=last])) }
		}
	}

	/// Returns the last segment of the path, if there is one, unless it is
	/// empty.
	///
	/// This does not consider the normalized version of the path, dot segments
	/// are preserved.
	#[inline]
	pub fn file_name(&self) -> Option<&Segment> {
		self.segments()
			.next_back()
			.map(|s| s)
			.filter(|s| !s.is_empty())
	}

	/// Returns the path without its final segment, if there is one.
	///
	/// ```
	/// # use iref::iri::Path;
	/// assert_eq!(Path::new("//foo").unwrap().parent().unwrap().as_str(), "/./");
	/// assert_eq!(Path::new("/foo").unwrap().parent().unwrap().as_str(), "/")
	/// ```
	#[inline]
	pub fn parent(&self) -> Option<&Self> {
		if self.is_empty() {
			None
		} else {
			let bytes = self.as_bytes();
			let mut end = bytes.len() - 1;

			loop {
				if bytes[end] == b'/' {
					if end == 0 {
						return Some(Self::EMPTY_ABSOLUTE);
					}

					break;
				}

				if end == 0 {
					return None;
				}

				end -= 1;
			}

			if end == 1 && bytes[0] == b'/' && bytes[1] == b'/' {
				// Ambiguous case `//foo` where returning the parent literally
				// would mean returning `/`, dropping the empty path.
				// Instead we return `/./`.
				unsafe { Some(Self::new_unchecked("/./")) }
			} else {
				unsafe {
					Some(Self::new_unchecked(std::str::from_utf8_unchecked(
						&bytes[..end],
					)))
				}
			}
		}
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
		if self.is_absolute() != prefix.is_absolute() {
			return None;
		}

		let mut buf = PathBuf::default();
		let mut self_it = self.normalized_segments();
		let mut prefix_it = prefix.normalized_segments();

		loop {
			match (self_it.next(), prefix_it.next()) {
				(Some(self_seg), Some(prefix_seg))
					if self_seg.as_pct_str() == prefix_seg.as_pct_str() => {}
				(_, Some(_)) => return None,
				(Some(seg), None) => buf.as_path_mut().push(seg),
				(None, None) => break,
			}
		}

		Some(buf)
	}
}

impl fmt::Display for Path {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for Path {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
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
					(Some(a), Some(b)) => match a.cmp(&b) {
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

pub enum Segments<'a> {
	Empty,
	NonEmpty {
		path: &'a Path,
		offset: usize,
		back_offset: usize,
	},
}

impl<'a> Iterator for Segments<'a> {
	type Item = &'a Segment;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Empty => None,
			Self::NonEmpty {
				path,
				offset,
				back_offset,
			} => {
				if offset < back_offset {
					match unsafe { path.next_segment_from(*offset) } {
						Some((segment, i)) => {
							*offset = i;
							Some(segment)
						}
						None => None,
					}
				} else {
					None
				}
			}
		}
	}
}

impl<'a> DoubleEndedIterator for Segments<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Segments::Empty => None,
			Segments::NonEmpty {
				path,
				offset,
				back_offset,
			} => {
				if offset < back_offset {
					match unsafe { path.previous_segment_from(*back_offset) } {
						Some((segment, i)) => {
							*back_offset = i;
							Some(segment)
						}
						None => None,
					}
				} else {
					None
				}
			}
		}
	}
}

/// Stack size (in number of `&Segment`) allocated for [`NormalizedSegments`] to
/// normalize a `Path`. If it needs more space, it will allocate memory on the
/// heap.
const NORMALIZE_STACK_SIZE: usize = 16;

pub struct NormalizedSegments<'a>(smallvec::IntoIter<[&'a Segment; NORMALIZE_STACK_SIZE]>);

impl<'a> NormalizedSegments<'a> {
	fn new(path: &'a Path) -> NormalizedSegments {
		let mut stack = SmallVec::<[&'a Segment; NORMALIZE_STACK_SIZE]>::new();

		for segment in path.segments() {
			match segment.as_bytes() {
				b"." => (),
				b".." => {
					if stack.last().map(|s| *s == Segment::PARENT).unwrap_or(true) {
						stack.push(segment)
					} else {
						stack.pop();
					}
				}
				_ => stack.push(segment),
			}
		}

		NormalizedSegments(stack.into_iter())
	}
}

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
