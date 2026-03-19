use core::{
	cell::Cell,
	cmp::Ordering,
	hash::{Hash, Hasher},
};

mod segment;
pub use segment::*;

#[cfg(feature = "std")]
mod r#mut;
#[cfg(feature = "std")]
pub use r#mut::*;

const CURRENT_SEGMENT: &[u8] = b".";

const PARENT_SEGMENT: &[u8] = b"..";

/// URI path component.
///
/// A path is a sequence of segments separated by `/`. A path can be absolute
/// (starting with `/`) or relative.
///
/// # Example
///
/// ```rust
/// use iref::uri::Path;
///
/// let path = Path::new("/foo/bar/baz").unwrap();
///
/// assert!(path.is_absolute());
/// assert_eq!(path.segment_count(), 3);
///
/// let segments: Vec<_> = path.segments().map(|s| s.as_str()).collect();
/// assert_eq!(segments, vec!["foo", "bar", "baz"]);
/// ```
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(super::grammar::Path)]
#[newtype(ord([u8], &[u8], str, &str))]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String), owned(PathBuf, derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Path(str);

impl Default for &Path {
	fn default() -> Self {
		Path::EMPTY_RELATIVE
	}
}

impl Path {
	/// The empty relative path.
	pub const EMPTY_RELATIVE: &'static Self = unsafe { Self::new_unchecked("") };

	/// The empty absolute path `/`.
	pub const EMPTY_ABSOLUTE: &'static Self = unsafe { Self::new_unchecked("/") };

	/// Returns the byte length of the path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert_eq!(Path::new("/foo/bar").unwrap().len(), 8);
	/// assert_eq!(Path::EMPTY_RELATIVE.len(), 0);
	/// ```
	pub fn len(&self) -> usize {
		self.as_bytes().len()
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path has no segments.
	/// The absolute path `/` is empty.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert!(Path::EMPTY_RELATIVE.is_empty());
	/// assert!(Path::EMPTY_ABSOLUTE.is_empty());
	/// assert!(!Path::new("/foo").unwrap().is_empty());
	/// ```
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.as_bytes().is_empty() || self.as_bytes() == *b"/"
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert!(Path::new("/foo/bar").unwrap().is_absolute());
	/// assert!(!Path::new("foo/bar").unwrap().is_absolute());
	/// ```
	#[inline(always)]
	pub fn is_absolute(&self) -> bool {
		self.as_bytes().starts_with(b"/")
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert!(Path::new("foo/bar").unwrap().is_relative());
	/// assert!(!Path::new("/foo/bar").unwrap().is_relative());
	/// ```
	#[inline(always)]
	pub fn is_relative(&self) -> bool {
		!self.is_absolute()
	}

	/// Returns the number of segments in the path.
	///
	/// This computes in linear time w.r.t the number of segments. It is
	/// equivalent to `path.segments().count()`.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert_eq!(Path::new("/foo/bar/baz").unwrap().segment_count(), 3);
	/// assert_eq!(Path::EMPTY_RELATIVE.segment_count(), 0);
	/// assert_eq!(Path::EMPTY_ABSOLUTE.segment_count(), 0);
	/// ```
	#[inline]
	pub fn segment_count(&self) -> usize {
		if self.is_empty() {
			0
		} else {
			1 + self.as_bytes()[1..].iter().filter(|&&b| b == b'/').count()
		}
	}

	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() { 1 } else { 0 }
	}

	/// Returns the first segment of the path, if any.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert_eq!(Path::new("/foo/bar").unwrap().first().unwrap(), "foo");
	/// assert!(Path::EMPTY_RELATIVE.first().is_none());
	/// ```
	pub fn first(&self) -> Option<&Segment> {
		if self.is_empty() {
			None
		} else {
			Some(unsafe { self.segment_at(self.first_segment_offset()).0 })
		}
	}

	/// Returns the last segment of the path, if any.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert_eq!(Path::new("/foo/bar").unwrap().last().unwrap(), "bar");
	/// assert!(Path::EMPTY_RELATIVE.last().is_none());
	/// ```
	pub fn last(&self) -> Option<&Segment> {
		if self.is_empty() {
			None
		} else {
			unsafe {
				self.previous_segment_from(self.as_bytes().len() + 1)
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

		(
			unsafe { Segment::new_unchecked_from_bytes(&bytes[offset..i]) },
			i + 1,
		)
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
			Some(unsafe { self.segment_at(offset) })
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
				Some((unsafe { self.segment_at(j) }.0, j))
			} else {
				Some((unsafe { self.segment_at(first_offset) }.0, first_offset))
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
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// let path = Path::new("/foo/bar/baz").unwrap();
	/// let segments: Vec<_> = path.segments().map(|s| s.as_str()).collect();
	/// assert_eq!(segments, vec!["foo", "bar", "baz"]);
	/// ```
	#[inline]
	pub fn segments(&self) -> Segments<'_> {
		if self.is_empty() {
			Segments::Empty
		} else {
			Segments::NonEmpty {
				path: self,
				offset: self.first_segment_offset(),
				back_offset: self.as_bytes().len() + 1,
				consumed: 0,
				total: Cell::new(None),
			}
		}
	}

	/// Iterate over the normalized segments of the path.
	///
	/// Remove the special dot segments `..` and `.` from the iteration using
	/// the usual path semantics for dot segments. This may be expensive for
	/// large paths since it will need to internally normalize the path first.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// let path = Path::new("/foo/bar/../baz").unwrap();
	/// let segments: Vec<_> = path.normalized_segments().map(|s| s.as_str()).collect();
	/// assert_eq!(segments, vec!["foo", "baz"]);
	/// ```
	#[inline]
	pub fn normalized_segments(&self) -> NormalizedSegments<'_> {
		NormalizedSegments::new(self)
	}

	/// Returns a normalized copy of the path.
	///
	/// Resolves `.` and `..` segments using the usual path semantics.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// let path = Path::new("/foo/bar/../baz/./qux").unwrap();
	/// assert_eq!(path.normalized(), "/foo/baz/qux");
	/// ```
	#[inline]
	#[cfg(feature = "std")]
	pub fn normalized(&self) -> PathBuf {
		let mut result: PathBuf = if self.is_absolute() {
			Self::EMPTY_ABSOLUTE.to_owned()
		} else {
			Self::EMPTY_RELATIVE.to_owned()
		};

		let mut open = false;
		for segment in self.segments() {
			open = result.as_path_mut().push_inner(segment)
		}

		if open && !result.is_empty() {
			result.as_path_mut().lazy_push(Segment::EMPTY);
		}

		result
	}

	/// Returns the last segment of the path, if there is one, unless it is
	/// empty.
	///
	/// This does not consider the normalized version of the path, dot segments
	/// are preserved.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert_eq!(Path::new("/foo/bar").unwrap().file_name().unwrap(), "bar");
	/// assert!(Path::new("/foo/bar/").unwrap().file_name().is_none());
	/// ```
	#[inline]
	pub fn file_name(&self) -> Option<&Segment> {
		self.segments().next_back().filter(|s| !s.is_empty())
	}

	/// Returns the directory path, which is the path without the file name.
	///
	/// # Example
	///
	/// ```
	/// # use iref::uri::Path;
	/// assert_eq!(Path::new("/foo/bar").unwrap().directory(), "/foo/");
	/// assert_eq!(Path::new("/foo").unwrap().directory(), "/");
	/// assert_eq!(Path::new("//foo").unwrap().directory(), "//");
	/// assert_eq!(Path::new("/").unwrap().directory(), "/");
	/// ```
	pub fn directory(&self) -> &Self {
		let bytes = self.as_bytes();
		if bytes.is_empty() {
			self
		} else {
			let mut i = bytes.len() - 1;

			while i > 0 && bytes[i] != b'/' {
				i -= 1
			}

			if i == 0 && bytes[i] != b'/' {
				Self::EMPTY_RELATIVE
			} else {
				unsafe { Self::new_unchecked_from_bytes(&bytes[..=i]) }
			}
		}
	}

	/// Returns the path without its final segment, if there is one.
	///
	/// ```
	/// # use iref::uri::Path;
	/// assert_eq!(Path::new("/foo/bar").unwrap().parent().unwrap(), "/foo");
	/// assert_eq!(Path::new("/foo").unwrap().parent().unwrap(), "/");
	/// assert_eq!(Path::new("//foo").unwrap().parent().unwrap(), "/./");
	/// assert_eq!(Path::new("/").unwrap().parent(), None);
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
				unsafe { Some(Self::new_unchecked_from_bytes(b"/./")) }
			} else {
				unsafe { Some(Self::new_unchecked_from_bytes(&bytes[..end])) }
			}
		}
	}

	/// Returns the path without its final segment, if there is one.
	///
	/// ```
	/// # use iref::uri::Path;
	/// assert_eq!(Path::new("/foo/bar").unwrap().parent_or_empty(), "/foo");
	/// assert_eq!(Path::new("/foo").unwrap().parent_or_empty(), "/");
	/// assert_eq!(Path::new("//foo").unwrap().parent_or_empty(), "/./");
	/// assert_eq!(Path::new("/").unwrap().parent_or_empty(), "/");
	/// assert_eq!(Path::new("").unwrap().parent_or_empty(), "");
	/// ```
	#[inline]
	pub fn parent_or_empty(&self) -> &Self {
		self.parent().unwrap_or_else(|| {
			if self.is_absolute() {
				Self::EMPTY_ABSOLUTE
			} else {
				Self::EMPTY_RELATIVE
			}
		})
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
	#[cfg(feature = "std")]
	pub fn suffix(&self, prefix: &Self) -> Option<PathBuf> {
		if self.is_absolute() != prefix.is_absolute() {
			return None;
		}

		let mut buf: PathBuf = Default::default();
		let mut self_it = self.normalized_segments();
		let mut prefix_it = prefix.normalized_segments();

		loop {
			match (self_it.next(), prefix_it.next()) {
				(Some(self_seg), Some(prefix_seg))
					if self_seg.as_pct_str() == prefix_seg.as_pct_str() => {}
				(_, Some(_)) => return None,
				(Some(seg), None) => {
					buf.as_path_mut().lazy_push(seg);
				}
				(None, None) => break,
			}
		}

		Some(buf)
	}

	/// Checks if this path looks like a scheme.
	///
	/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
	/// valid scheme, or `false` otherwise.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Path;
	///
	/// assert!(Path::new("http:foo").unwrap().looks_like_scheme());
	/// assert!(!Path::new("foo/bar").unwrap().looks_like_scheme());
	/// ```
	pub fn looks_like_scheme(&self) -> bool {
		crate::common::parse::looks_like_scheme(self.as_bytes())
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
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Path {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering {
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

impl Hash for Path {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
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
		consumed: usize,
		total: Cell<Option<usize>>,
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
				consumed,
				..
			} => {
				if offset < back_offset {
					match unsafe { path.next_segment_from(*offset) } {
						Some((segment, i)) => {
							*offset = i;
							*consumed += 1;
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

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = match self {
			Self::Empty => 0,
			Self::NonEmpty {
				path,
				consumed,
				total,
				..
			} => {
				let t = match total.get() {
					Some(t) => t,
					None => {
						let t = path.segment_count();
						total.set(Some(t));
						t
					}
				};
				t - *consumed
			}
		};
		(len, Some(len))
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
				consumed,
				..
			} => {
				if offset < back_offset {
					match unsafe { path.previous_segment_from(*back_offset) } {
						Some((segment, i)) => {
							*back_offset = i;
							*consumed += 1;
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

impl<'a> ExactSizeIterator for Segments<'a> {}

/// Stack size (in number of `&Segment`) allocated for [`NormalizedSegments`] to
/// normalize a `Path`. If it needs more space, it will allocate memory on the
/// heap.
const NORMALIZE_STACK_SIZE: usize = 16;

pub struct NormalizedSegments<'a>(smallvec::IntoIter<[&'a Segment; NORMALIZE_STACK_SIZE]>);

impl<'a> NormalizedSegments<'a> {
	fn new(path: &'a Path) -> NormalizedSegments<'a> {
		let relative = path.is_relative();
		let mut stack = smallvec::SmallVec::<[&'a Segment; NORMALIZE_STACK_SIZE]>::new();

		let mut open = false;
		for segment in path.segments() {
			open = match segment.as_bytes() {
				CURRENT_SEGMENT => true,
				PARENT_SEGMENT => {
					if stack
						.last()
						.map(|s| s.as_bytes() == PARENT_SEGMENT)
						.unwrap_or(relative)
					{
						stack.push(segment)
					} else {
						stack.pop();
					};

					true
				}
				_ => {
					stack.push(segment);
					false
				}
			};
		}

		if open && !stack.is_empty() {
			stack.push(Segment::EMPTY);
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

#[cfg(feature = "std")]
impl PathBuf {
	/// Builds a path from the given segments.
	///
	/// If `absolute` is `true`, the path will start with `/`.
	/// Each segment is appended using [`push`](Self::push), which
	/// interprets `.` and `..` segments.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, Segment};
	///
	/// let abs = PathBuf::from_segments(true, [
	///     Segment::new("a").unwrap(),
	///     Segment::new("b").unwrap(),
	///     Segment::new("c").unwrap(),
	/// ]);
	/// assert_eq!(abs, "/a/b/c");
	///
	/// let rel = PathBuf::from_segments(false, [
	///     Segment::new("a").unwrap(),
	///     Segment::new("b").unwrap(),
	///     Segment::new("c").unwrap(),
	/// ]);
	/// assert_eq!(rel, "a/b/c");
	/// ```
	pub fn from_segments<'a>(
		absolute: bool,
		segments: impl IntoIterator<Item = &'a Segment>,
	) -> Self {
		let mut path = if absolute {
			Path::EMPTY_ABSOLUTE
		} else {
			Path::EMPTY_RELATIVE
		}
		.to_owned();

		for segment in segments {
			path.push(segment);
		}

		path
	}

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
		unsafe { self.0.as_mut_vec() }
	}

	/// Returns a mutable path reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().clear();
	/// assert_eq!(path, "/");
	/// ```
	pub fn as_path_mut(&mut self) -> PathMut<'_> {
		PathMut::from_path(self)
	}

	/// Pushes a segment to the path without `.` and `..` semantics.
	///
	/// Simply appends the segment without resolving special segments.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, Segment};
	///
	/// let mut path = PathBuf::new("/foo".to_string()).unwrap();
	/// path.lazy_push(Segment::new("..").unwrap());
	/// assert_eq!(path, "/foo/..");
	///
	/// // It is possible to push empty segments.
	/// path.lazy_push(Segment::new("").unwrap());
	/// assert_eq!(path, "/foo/../");
	/// path.lazy_push(Segment::new("").unwrap());
	/// assert_eq!(path, "/foo/..//");
	/// ```
	pub fn lazy_push(&mut self, segment: &Segment) -> &mut Self {
		self.as_path_mut().lazy_push(segment);
		self
	}

	/// Adds a segment at the end of the path.
	///
	/// Same as [`Self::lazy_push`] but accepts a `&str` instead of a
	/// [`&Segment`](super::Segment). Returns an error if the input
	/// string is not a valid path segment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo".to_string()).unwrap();
	/// path.try_lazy_push("bar").unwrap();
	/// assert_eq!(path, "/foo/bar");
	/// ```
	pub fn try_lazy_push<'s>(
		&mut self,
		segment: &'s str,
	) -> Result<&mut Self, InvalidSegment<&'s str>> {
		self.as_path_mut().try_lazy_push(segment)?;
		Ok(self)
	}

	/// Push the given segment to this path using the `.` and `..` segments semantics.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, Segment};
	///
	/// let mut path = PathBuf::new("/foo".to_string()).unwrap();
	/// path.push(Segment::new("bar").unwrap());
	/// assert_eq!(path, "/foo/bar");
	///
	/// path.push(Segment::new("..").unwrap());
	/// assert_eq!(path, "/foo/");
	/// ```
	#[inline]
	pub fn push(&mut self, segment: &Segment) -> &mut Self {
		self.as_path_mut().push(segment);
		self
	}

	/// Pushes the given segment to this path using the `.` and `..` segments
	/// semantics.
	///
	/// Same as [`Self::push`] but accepts a `&str` instead of
	/// a [`&Segment`](super::Segment). Returns an error if the input
	/// string is not a valid path segment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_push("..").unwrap();
	/// assert_eq!(path, "/foo/");
	/// ```
	#[inline]
	pub fn try_push<'s>(&mut self, segment: &'s str) -> Result<&mut Self, InvalidSegment<&'s str>> {
		self.as_path_mut().try_push(segment)?;
		Ok(self)
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{Path, PathBuf};
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.append(Path::new("../baz").unwrap());
	/// assert_eq!(path, "/foo/baz");
	/// ```
	#[inline]
	pub fn append<'s, P: IntoIterator<Item = &'s Segment>>(&mut self, path: P) -> &mut Self {
		self.as_path_mut().append(path);
		self
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.try_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_append(["baz", "qux"]).unwrap();
	/// assert_eq!(path, "/foo/bar/baz/qux");
	/// ```
	#[inline]
	pub fn try_append<'s, P: IntoIterator<Item = &'s str>>(
		&mut self,
		path: P,
	) -> Result<&mut Self, InvalidSegment<&'s str>> {
		self.as_path_mut().try_append(path)?;
		Ok(self)
	}

	/// Joins this path to the given path.
	///
	/// If the input path is absolute, this is equivalent to
	/// [`Self::replace`]. If the input path is relative, this is
	/// equivalent to [`Self::append`].
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{Path, PathBuf};
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().join(Path::new("baz/qux").unwrap());
	/// assert_eq!(path, "/foo/bar/baz/qux");
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().join(Path::new("/baz").unwrap());
	/// assert_eq!(path, "/baz");
	/// ```
	pub fn join(&mut self, path: &Path) -> &mut Self {
		self.as_path_mut().join(path);
		self
	}

	/// Joins this path to the given path.
	///
	/// Same as [`Self::join`] but accepts a `&str` instead of a
	/// [`&Path`](Path). Returns an error if the input string is not
	/// a valid path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_join("baz/qux").unwrap();
	/// assert_eq!(path, "/foo/bar/baz/qux");
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_join("/baz").unwrap();
	/// assert_eq!(path, "/baz");
	/// ```
	pub fn try_join<'s>(&mut self, path: &'s str) -> Result<&mut Self, InvalidPath<&'s str>> {
		self.as_path_mut().try_join(path)?;
		Ok(self)
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead. Returns `true` if a segment was popped.
	///
	/// Has no effect if the path is empty and absolute.
	/// Use [`Self::try_pop`] if you need to know if the path has been modified.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.pop();
	/// assert_eq!(path, "/foo");
	///
	/// let mut path = PathBuf::new("foo/bar".to_string()).unwrap();
	/// path.pop().pop().pop();
	/// assert_eq!(path, "..");
	/// ```
	pub fn pop(&mut self) -> &mut Self {
		self.as_path_mut().try_pop();
		self
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead.
	///
	/// Returns `true` if the path has been modified, or `false` otherwise.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_pop();
	/// assert_eq!(path, "/foo");
	/// ```
	pub fn try_pop(&mut self) -> bool {
		self.as_path_mut().try_pop()
	}

	/// Clears the path, removing all segments.
	///
	/// Keeps the path absolute if it was absolute.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.clear();
	/// assert_eq!(path, "/");
	/// ```
	pub fn clear(&mut self) -> &mut Self {
		self.as_path_mut().clear();
		self
	}

	/// Replaces this path with the given path.
	///
	/// Handles ambiguities that may arise during replacement.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{Path, PathBuf};
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.replace(Path::new("/baz/qux").unwrap());
	/// assert_eq!(path, "/baz/qux");
	/// ```
	pub fn replace(&mut self, path: &Path) -> &mut Self {
		self.as_path_mut().replace(path);
		self
	}

	/// Replaces this path with the given path.
	///
	/// Same as [`Self::replace`] but accepts a `&str` instead of a
	/// [`&Path`](Path). Returns an error if the input string is not
	/// a valid path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.try_replace("/baz/qux").unwrap();
	/// assert_eq!(path, "/baz/qux");
	/// ```
	pub fn try_replace<'p>(&mut self, path: &'p str) -> Result<&mut Self, InvalidPath<&'p str>> {
		self.as_path_mut().try_replace(path)?;
		Ok(self)
	}

	/// Normalizes the path in place.
	///
	/// Resolves `.` and `..` segments using the usual path semantics.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar/../baz".to_string()).unwrap();
	/// path.normalize();
	/// assert_eq!(path, "/foo/baz");
	/// ```
	#[inline]
	pub fn normalize(&mut self) -> &mut Self {
		self.as_path_mut().normalize();
		self
	}
}

#[cfg(feature = "std")]
impl<'a> FromIterator<&'a Segment> for PathBuf {
	fn from_iter<I: IntoIterator<Item = &'a Segment>>(iter: I) -> Self {
		let mut path = PathBuf::default();
		for segment in iter {
			path.push(segment);
		}
		path
	}
}

/// Parses a URI [`Path`] at compile time.
#[macro_export]
macro_rules! path {
	($value:literal) => {
		match $crate::uri::Path::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI path"),
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let path = Path::EMPTY_RELATIVE;
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
		let path = Path::new(b"a/b").unwrap();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "a");
		assert!(segments.next().unwrap().as_str() == "b");
		assert!(segments.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let path = Path::new(b"/foo/bar").unwrap();
		assert!(!path.is_empty());
		assert!(path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_bytes() == b"foo");
		assert!(segments.next().unwrap().as_bytes() == b"bar");
		assert!(segments.next().is_none());
	}

	#[test]
	fn next_segment() {
		let vectors: [(&[u8], usize, Option<(&[u8], usize)>); 6] = [
			(b"foo/bar", 0, Some((b"foo", 4))),
			(b"foo/bar", 4, Some((b"bar", 8))),
			(b"foo/bar", 8, None),
			(b"foo/bar/", 8, Some((b"", 9))),
			(b"foo/bar/", 9, None),
			(b"//foo", 1, Some((b"", 2))),
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
		let vectors: [(&[u8], usize, Option<(&[u8], usize)>); 7] = [
			(b"/foo/bar", 1, None),
			(b"foo/bar", 0, None),
			(b"foo/bar", 4, Some((b"foo", 0))),
			(b"foo/bar", 8, Some((b"bar", 4))),
			(b"foo/bar/", 8, Some((b"bar", 4))),
			(b"foo/bar/", 9, Some((b"", 8))),
			(b"//a/b", 4, Some((b"a", 2))),
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
		let vectors: [(&[u8], Option<&[u8]>); 4] = [
			(b"", None),
			(b"/", None),
			(b"//", Some(b"")),
			(b"/foo/bar", Some(b"foo")),
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
		let vectors: [(&[u8], &[&[u8]]); 8] = [
			(b"", &[]),
			(b"foo", &[b"foo"]),
			(b"/foo", &[b"foo"]),
			(b"foo/", &[b"foo", b""]),
			(b"/foo/", &[b"foo", b""]),
			(b"a/b/c/d", &[b"a", b"b", b"c", b"d"]),
			(b"a/b//c/d", &[b"a", b"b", b"", b"c", b"d"]),
			(
				b"//a/b/foo//bar/",
				&[b"", b"a", b"b", b"foo", b"", b"bar", b""],
			),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected)
					.all(|(a, b)| a.as_bytes() == *b)
			)
		}
	}

	#[test]
	fn segments_rev() {
		let vectors: [(&[u8], &[&[u8]]); 8] = [
			(b"", &[]),
			(b"foo", &[b"foo"]),
			(b"/foo", &[b"foo"]),
			(b"foo/", &[b"foo", b""]),
			(b"/foo/", &[b"foo", b""]),
			(b"a/b/c/d", &[b"a", b"b", b"c", b"d"]),
			(b"a/b//c/d", &[b"a", b"b", b"", b"c", b"d"]),
			(
				b"//a/b/foo//bar/",
				&[b"", b"a", b"b", b"foo", b"", b"bar", b""],
			),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().rev().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected.into_iter().rev())
					.all(|(a, b)| a.as_bytes() == *b)
			)
		}
	}

	#[test]
	fn normalized() {
		let vectors: [(&[u8], &[u8]); 9] = [
			(b"", b""),
			(b"a/b/c", b"a/b/c"),
			(b"a/..", b""),
			(b"a/b/..", b"a/"),
			(b"a/b/../", b"a/"),
			(b"a/b/c/..", b"a/b/"),
			(b"a/b/c/.", b"a/b/c/"),
			(b"a/../..", b"../"),
			(b"/a/../..", b"/"),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let output = path.normalized();
			assert_eq!(output.as_bytes(), expected);
		}
	}

	#[test]
	fn eq() {
		let vectors: [(&str, &str); _] = [
			("a/b/c", "a/b/c"),
			("a/b/c/", "a/b/c/."),
			("a/b/c/", "a/b/c/./"),
			("a/b/c", "a/b/../b/c"),
			("a/b/c/..", "a/b/"),
			("a/..", ""),
			("/a/..", "/"),
			("a/../../", "../"),
			("/a/../../", "/../"),
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
		let vectors: [(&str, &str); _] = [
			("a/b/c", "a/b/c/"),
			("a/b/c", "a/b/c/."),
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
		let vectors: [(&[u8], Option<&[u8]>); 2] = [
			(b"//a/b/foo//bar/", None),
			(b"//a/b/foo//bar", Some(b"bar")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.file_name().map(|s| s.as_bytes()), expected)
		}
	}

	#[test]
	fn parent() {
		let vectors: [(&[u8], Option<&[u8]>); 11] = [
			(b"", None),
			(b"/", None),
			(b".", None),
			(b"//a/b/foo//bar", Some(b"//a/b/foo/")),
			(b"//a/b/foo//", Some(b"//a/b/foo/")),
			(b"//a/b/foo/", Some(b"//a/b/foo")),
			(b"//a/b/foo", Some(b"//a/b")),
			(b"//a/b", Some(b"//a")),
			(b"//a", Some(b"/./")),
			(b"/./", Some(b"/.")),
			(b"/.", Some(b"/")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.parent().map(Path::as_bytes), expected)
		}
	}

	#[test]
	fn suffix() {
		let vectors: [(&str, &str, Option<&str>); _] = [
			("/foo/bar/baz", "/foo/bar", Some("baz")),
			("//foo", "/", Some(".//foo")),
			("/a/b/baz", "/foo/bar", None),
			("/foo/bar/baz", "/foo", Some("bar/baz")),
		];

		for (path, prefix, expected_suffix) in vectors {
			let path = Path::new(path).unwrap();
			let suffix = path.suffix(Path::new(prefix).unwrap());
			assert_eq!(suffix.as_deref().map(Path::as_str), expected_suffix)
		}
	}
}
