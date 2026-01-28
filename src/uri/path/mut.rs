use core::fmt;
use std::ops::{Deref, Range};

use super::{InvalidPath, InvalidSegment, Path, PathBuf, Segment};

const CURRENT_SEGMENT: &[u8] = b".";

const PARENT_SEGMENT: &[u8] = b"..";

/// Stack size (in bytes) allocated for the [`PathMut::normalize`] method. If it
/// needs more space, it will allocate memory on the heap.
const NORMALIZE_IN_PLACE_BUFFER_LEN: usize = 512;

/// Mutable URI path reference.
///
/// This type allows in-place modification of a URI path within a larger
/// buffer, handling ambiguities that may arise during path manipulation.
///
/// # Example
///
/// ```rust
/// use iref::uri::PathBuf;
///
/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
/// path.as_path_mut().pop();
/// assert_eq!(path, "/foo");
/// ```
pub struct PathMut<'a> {
	/// Arbitrary byte buffer containing the path.
	buffer: &'a mut Vec<u8>,

	/// Path range.
	range: Range<usize>,

	/// Determines if the path follows an authority part,
	/// in which case some disambiguation rules applies.
	follows_authority: bool,
}

impl<'a> Deref for PathMut<'a> {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		self.as_path()
	}
}

impl<'a> PathMut<'a> {
	/// Creates a new mutable path reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathMut;
	///
	/// let mut buffer = b"/foo/bar".to_vec();
	/// let path_mut = PathMut::new(&mut buffer, 0..8).unwrap();
	/// assert_eq!(path_mut.as_path(), "/foo/bar");
	/// ```
	pub fn new(
		buffer: &'a mut Vec<u8>,
		range: Range<usize>,
	) -> Result<Self, InvalidPath<&'a [u8]>> {
		if Path::validate_bytes(&buffer[range.clone()]) {
			Ok(unsafe { Self::new_unchecked(buffer, range) })
		} else {
			Err(InvalidPath(buffer))
		}
	}

	/// Creates a new mutable path reference.
	///
	/// # Safety
	///
	/// The buffer content between in the range `start..end` must be a valid
	/// IRI path.
	pub unsafe fn new_unchecked(buffer: &'a mut Vec<u8>, range: Range<usize>) -> Self {
		let follows_authority =
			crate::common::parse::find_authority(&buffer[..range.start], 0).is_ok();

		Self {
			buffer,
			range,
			follows_authority,
		}
	}

	/// Creates a mutable path reference from a path buffer.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, PathMut};
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// let path_mut = PathMut::from_path(&mut path);
	/// assert_eq!(path_mut.as_path(), "/foo/bar");
	/// ```
	pub fn from_path(path: &'a mut PathBuf) -> Self {
		let buffer = unsafe {
			// Safe because `PathMut` preserves well formed paths.
			path.as_mut_vec()
		};
		let end = buffer.len();

		Self {
			buffer,
			range: 0..end,
			follows_authority: false,
		}
	}

	/// Creates a mutable path reference from a path buffer, assuming it
	/// follows an authority.
	///
	/// This is mostly for testing.
	///
	/// # Panic
	///
	/// This function will panic if the path is relative and non-empty, since
	/// such path cannot follow an authority.
	pub fn from_path_following_authority(path: &'a mut PathBuf) -> Self {
		assert!(path.is_absolute() || path.is_empty());

		let buffer = unsafe {
			// Safe because `PathMut` preserves well formed paths.
			path.as_mut_vec()
		};
		let end = buffer.len();

		Self {
			buffer,
			range: 0..end,
			follows_authority: true,
		}
	}

	/// Returns the path as a reference.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// assert_eq!(path.as_path_mut().as_path(), "/foo/bar");
	/// ```
	pub fn as_path(&self) -> &Path {
		unsafe { Path::new_unchecked_from_bytes(&self.buffer[self.range.clone()]) }
	}

	/// Returns the byte index where the first path segment begins.
	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() {
			self.range.start + 1
		} else {
			self.range.start
		}
	}

	/// Add a segment at the end of the path.
	///
	/// Same as [`Self::push`] but does not interpret the `.` and `..`
	/// segments. They will be added literally to the path.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, Segment};
	///
	/// let mut path = PathBuf::new("/foo".to_string()).unwrap();
	/// path.as_path_mut().lazy_push(Segment::new("bar").unwrap());
	/// assert_eq!(path, "/foo/bar");
	///
	/// // It is possible to push empty segments.
	/// path.as_path_mut().lazy_push(Segment::new("").unwrap());
	/// assert_eq!(path, "/foo/bar/");
	/// path.as_path_mut().lazy_push(Segment::new("").unwrap());
	/// assert_eq!(path, "/foo/bar//");
	/// ```
	pub fn lazy_push(&mut self, segment: &super::Segment) -> &mut Self {
		let absolutize = self.follows_authority && self.is_empty() && self.is_relative();

		// Disambiguate if the path is empty and one of the following is true:
		// - `segment` looks like a scheme and path is a the start.
		// - `segment` is empty, path is absolute and following an authority.
		// - `segment` is empty, path is relative.
		let disambiguate = self.is_empty()
			&& ((self.range.start == 0 && segment.looks_like_scheme()) || segment.is_empty());

		if absolutize {
			if disambiguate {
				// Make the path absolute, and disambiguate.
				let start = self.range.start;
				let len = 3 + segment.len();
				crate::utils::allocate_range(self.buffer, start..start, len);
				self.range.end += len;
				self.buffer[start..(start + 3)].copy_from_slice(b"/./");
				self.buffer[(start + 3)..self.range.end].copy_from_slice(segment.as_bytes());
			} else {
				// Make the path absolute.
				let start = self.range.start;
				let len = 1 + segment.len();
				crate::utils::allocate_range(self.buffer, start..start, len);
				self.range.end += len;
				self.buffer[start..(start + 1)].copy_from_slice(b"/");
				self.buffer[(start + 1)..self.range.end].copy_from_slice(segment.as_bytes());
			}
		} else if disambiguate {
			// Add `./` before the segment (to disambiguate).
			let start = self.first_segment_offset();
			let len = 2 + segment.len();
			crate::utils::allocate_range(self.buffer, start..start, len);
			self.range.end += len;
			let offset = start + 2;
			self.buffer[start..offset].copy_from_slice(b"./");
			self.buffer[offset..self.range.end].copy_from_slice(segment.as_bytes());
		} else if self.is_empty() {
			// Simply replace.
			crate::utils::replace(
				self.buffer,
				self.range.end..self.range.end,
				segment.as_bytes(),
			);
			self.range.end += segment.len();
		} else {
			// Append.
			let bytes = self.as_bytes();
			let mut start_offset = 0usize;
			if (self.follows_authority || bytes.len() > 3) && bytes.ends_with(b"/./") {
				// we can remove the `./` here.
				start_offset = 2;
			};

			let start = self.range.end - start_offset;
			let len = 1 + segment.len();
			crate::utils::allocate_range(self.buffer, start..self.range.end, len);

			self.buffer[start] = b'/';
			self.range.end += len - start_offset;
			let segment_offset = start + 1;
			self.buffer[segment_offset..self.range.end].copy_from_slice(segment.as_bytes());
		}

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
	/// path.as_path_mut().try_lazy_push("bar").unwrap();
	/// assert_eq!(path, "/foo/bar");
	/// ```
	pub fn try_lazy_push<'s>(
		&mut self,
		segment: &'s str,
	) -> Result<&mut Self, super::InvalidSegment<&'s str>> {
		self.lazy_push(segment.try_into()?);
		Ok(self)
	}

	/// Push the given segment to this path using the `.` and `..` segments
	/// semantics.
	///
	/// Returns wether or not a special segment has been pushed and
	/// should be followed by an empty segment when doing reference
	/// resolution.
	#[inline]
	pub(crate) fn push_inner(&mut self, segment: &super::Segment) -> bool {
		match segment.as_bytes() {
			CURRENT_SEGMENT => true,
			PARENT_SEGMENT => {
				self.pop();
				true
			}
			_ => {
				if !segment.is_empty() || !self.is_empty() {
					self.lazy_push(segment);
				}

				false
			}
		}
	}

	/// Adds a segment at the end of the path.
	///
	/// # Ambiguities
	///
	/// Adding a segment to an empty path may introduce ambiguities in several
	/// cases. Here is how this function deals with those cases.
	///
	/// ## Empty segment
	///
	/// Adding an empty segment on an empty path may add ambiguity in two
	/// cases:
	/// 1. if the path is relative, adding a `/` would make the path
	///    absolute (e.g. `scheme:` becomes `scheme:/`) ;
	/// 2. if the path is absolute adding a `/` would add two empty segments
	///    (e.g. `scheme:/` becomes `scheme://`), and it may be confused with an
	///    authority part ;
	///
	/// To avoid such ambiguity, in both cases this function will add a `.`
	/// segment to the path, preserving its semantics:
	/// 1. `scheme:` becomes `scheme:./` instead of `scheme:/` ;
	/// 2. `scheme:/` becomes `scheme:/./` instead of `scheme://`.
	///
	/// ## Relative empty path with authority
	///
	/// If the path is empty, but an authority is present, the path is turned
	/// absolute so the segment is not concatenated to the authority.
	///
	/// ## Segment containing a `:`
	///
	/// If the path does not follow a scheme and/or authority part, a `:` in
	/// the first segment may be confused with a scheme separator
	/// (e.g. `looks-like-a-scheme:rest`).
	/// To avoid such ambiguity, this function will add a `.` segment to the
	/// path, preserving its semantics (e.g. `./looks-like-a-scheme:rest`).
	///
	/// ## `.` and `..`
	///
	/// This method will interpret `.` and `..` such that pushing `.`
	/// has no effect, and `..` is equivalent to [`Self::pop`].
	/// Use [`Self::lazy_push`] to not interpret those segments.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{PathBuf, Segment};
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().push(Segment::new("..").unwrap());
	/// assert_eq!(path, "/foo/");
	/// ```
	#[inline]
	pub fn push(&mut self, segment: &super::Segment) -> &mut Self {
		if self.push_inner(segment) && !self.is_empty() {
			self.lazy_push(super::Segment::EMPTY)
		} else {
			self
		}
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
	/// path.as_path_mut().try_push("..").unwrap();
	/// assert_eq!(path, "/foo/");
	/// ```
	#[inline]
	pub fn try_push<'s>(
		&mut self,
		segment: &'s str,
	) -> Result<&mut Self, super::InvalidSegment<&'s str>> {
		Ok(self.push(segment.try_into()?))
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
	/// path.as_path_mut().append(Path::new("../baz").unwrap());
	/// assert_eq!(path, "/foo/baz");
	/// ```
	#[inline]
	pub fn append<'s, S: IntoIterator<Item = &'s Segment>>(&mut self, path: S) -> &mut Self {
		let mut open = false;
		for segment in path {
			open = self.push_inner(segment);
		}

		if open && !self.is_empty() {
			self.lazy_push(super::Segment::EMPTY)
		} else {
			self
		}
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.try_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	///
	/// Same as [`Self::append`], but accepts `&str` instead of
	/// [`&Segment`](super::Segment). Returns an error if one item is
	/// not a valid segment.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::PathBuf;
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().try_append(["baz", "qux"]).unwrap();
	/// assert_eq!(path, "/foo/bar/baz/qux");
	/// ```
	#[inline]
	pub fn try_append<'s, S: IntoIterator<Item = &'s str>>(
		&mut self,
		path: S,
	) -> Result<&mut Self, InvalidSegment<&'s str>> {
		let mut open = false;
		for segment in path {
			open = self.push_inner(segment.try_into()?);
		}

		if open && !self.is_empty() {
			Ok(self.lazy_push(Segment::EMPTY))
		} else {
			Ok(self)
		}
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
		if path.is_absolute() {
			self.replace(path)
		} else {
			self.append(path)
		}
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
	/// path.as_path_mut().try_join("baz/qux").unwrap();
	/// assert_eq!(path, "/foo/bar/baz/qux");
	///
	/// let mut path = PathBuf::new("/foo/bar".to_string()).unwrap();
	/// path.as_path_mut().try_join("/baz").unwrap();
	/// assert_eq!(path, "/baz");
	/// ```
	pub fn try_join<'s>(&mut self, path: &'s str) -> Result<&mut Self, InvalidPath<&'s str>> {
		self.join(Path::new(path)?);
		Ok(self)
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty and relative, or ends in `..`, then a `..` segment
	/// will be added instead.
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
	/// path.as_path_mut().pop();
	/// assert_eq!(path, "/foo");
	/// ```
	pub fn pop(&mut self) -> &mut Self {
		self.try_pop();
		self
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty and relative, or ends in `..`, then a `..` segment
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
	/// path.as_path_mut().try_pop();
	/// assert_eq!(path, "/foo");
	/// ```
	pub fn try_pop(&mut self) -> bool {
		let is_empty = self.is_empty();

		if (is_empty && self.is_relative()) || self.last() == Some(super::Segment::PARENT) {
			self.lazy_push(super::Segment::PARENT);
			true
		} else if !is_empty {
			let start = self.first_segment_offset();
			let mut i = self.range.end - 1;

			while i > start && self.buffer[i] != b'/' {
				i -= 1
			}

			crate::utils::replace(self.buffer, i..self.range.end, &[]);
			self.range.end = i;
			true
		} else {
			false
		}
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
	/// path.as_path_mut().clear();
	/// assert_eq!(path, "/");
	/// ```
	pub fn clear(&mut self) -> &mut Self {
		let start = self.first_segment_offset();
		crate::utils::replace(self.buffer, start..self.range.end, b"");
		self.range.end = start;
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
	/// path.as_path_mut().replace(Path::new("/baz/qux").unwrap());
	/// assert_eq!(path, "/baz/qux");
	/// ```
	pub fn replace(&mut self, path: &Path) -> &mut Self {
		let range = self.range.start..self.range.end;

		let has_authority = self.follows_authority;
		if !has_authority && path.as_bytes().starts_with(b"//") {
			// AMBIGUITY: The URI `http:old/path` would become
			//            `http://new_path`, but `//new_path` is not the
			//            authority.
			// SOLUTION:  We change `//new_path` to `/.//new_path`.
			let start = range.start;
			let actual_start = start + 2;
			crate::utils::allocate_range(self.buffer, range, path.len() + 2);
			self.buffer[start..actual_start].copy_from_slice(b"/.");
			self.buffer[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes());
			self.range.end = self.range.start + path.len() + 2;
		} else if has_authority && !path.is_empty() && path.is_relative() {
			// VALIDITY: When an authority is present, the path must be
			//           absolute, unless it is empty.
			let start = range.start;
			let actual_start = start + 1;
			crate::utils::allocate_range(self.buffer, range, path.len() + 1);
			self.buffer[start] = b'/';
			self.buffer[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes());
			self.range.end = self.range.start + path.len() + 1;
		} else if range.start == 0 && path.looks_like_scheme() {
			// AMBIGUITY: The URI `old/path` would become `new:path`, but `new`
			//            is not the scheme.
			// SOLUTION:  We change `new:path` to `./new:path`.
			let start = range.start;
			let actual_start = start + 2;
			crate::utils::allocate_range(self.buffer, range, path.len() + 2);
			self.buffer[start..actual_start].copy_from_slice(b"./");
			self.buffer[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes());
			self.range.end = self.range.start + path.len() + 2;
		} else {
			crate::utils::replace(self.buffer, range, path.as_bytes());
			self.range.end = self.range.start + path.len();
		}

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
	/// path.as_path_mut().try_replace("/baz/qux").unwrap();
	/// assert_eq!(path, "/baz/qux");
	/// ```
	pub fn try_replace<'p>(&mut self, path: &'p str) -> Result<&mut Self, InvalidPath<&'p str>> {
		self.replace(Path::new(path)?);
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
	/// path.as_path_mut().normalize();
	/// assert_eq!(path, "/foo/baz");
	/// ```
	#[inline]
	pub fn normalize(&mut self) -> &mut Self {
		let mut buffer: smallvec::SmallVec<[u8; NORMALIZE_IN_PLACE_BUFFER_LEN]> =
			smallvec::SmallVec::new();

		for (i, segment) in self.normalized_segments().enumerate() {
			if i > 0 {
				buffer.push(b'/')
			}

			buffer.extend_from_slice(segment.as_bytes())
		}

		let start = self.first_segment_offset();
		crate::utils::replace(self.buffer, start..self.range.end, &buffer);
		self.range.end = start + buffer.len();
		self
	}
}

impl<'a> fmt::Display for PathMut<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_path().fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::uri::{PathBuf, Segment};

	#[test]
	fn lazy_push() {
		let vectors: [(&str, &str, &str); _] = [
			("", "foo", "foo"),
			("/", "foo", "/foo"),
			("", "", "./"),
			("/", "", "/./"),
			("foo", "bar", "foo/bar"),
			("/foo", "bar", "/foo/bar"),
			("foo", "", "foo/"),
			("foo/bar", "", "foo/bar/"),
			("foo/", "", "foo//"),
			("a/b/c", "d", "a/b/c/d"),
			("/a/b/c", "d", "/a/b/c/d"),
			("a/b/c/", "d", "a/b/c//d"),
		];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_owned()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.lazy_push(segment);
			assert_eq!(path_mut.as_str(), expected)
		}
	}

	#[test]
	fn lazy_push_following_authority() {
		let vectors: [(&[u8], &[u8], &[u8]); _] = [
			(b"", b"foo", b"/foo"),
			(b"/", b"foo", b"/foo"),
			(b"", b"", b"/./"),
			(b"/", b"", b"/./"),
			(b"/foo", b"bar", b"/foo/bar"),
			(b"/a/b/c", b"d", b"/a/b/c/d"),
		];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path_following_authority(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.lazy_push(segment);
			assert_eq!(path_mut.as_bytes(), expected)
		}
	}

	#[test]
	fn push() {
		let vectors: [(&str, &str, &str); _] =
			[("foo/bar", "..", "foo/"), ("foo/bar", ".", "foo/bar/")];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_owned()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.push(segment);
			assert_eq!(path_mut.as_str(), expected)
		}
	}

	#[test]
	fn append() {
		let vectors: [(&str, &str, &str); _] = [("foo/bar", "..", "foo/")];

		for (a, b, expected) in vectors {
			let mut a = PathBuf::new(a.to_owned()).unwrap();
			let mut a_mut = PathMut::from_path(&mut a);
			let b = Path::new(b).unwrap();
			a_mut.append(b.segments());
			assert_eq!(a_mut.as_str(), expected)
		}
	}

	#[test]
	fn replace() {
		let vectors = [
			("a", "foo", "foo"),
			("a/b", "//foo", "/.//foo"), // AMBIGUITY: may be confused with an authority.
			("a/b/c", "foo:bar", "./foo:bar"), // AMBIGUITY: may be confused with a scheme.
			("../", "/foo:bar", "/foo:bar"),
		];

		for (a, b, expected) in vectors {
			let mut path = PathBuf::new(a.to_owned()).unwrap();
			PathMut::from_path(&mut path).try_replace(b).unwrap();
			assert_eq!(path.as_str(), expected)
		}
	}

	#[test]
	fn replace_following_authority() {
		let vectors = [
			("", "foo", "/foo"), // AMBIGUITY: may be confused with (part of) the authority
			("/", "//foo", "//foo"),
		];

		for (a, b, expected) in vectors {
			let mut path = PathBuf::new(a.to_owned()).unwrap();
			PathMut::from_path_following_authority(&mut path)
				.try_replace(b)
				.unwrap();
			assert_eq!(path.as_str(), expected)
		}
	}

	#[test]
	fn pop() {
		let vectors: [(&[u8], &[u8]); 6] = [
			(b"", b".."),
			(b"/", b"/"),
			(b"/..", b"/../.."),
			(b"foo", b""),
			(b"foo/bar", b"foo"),
			(b"foo/bar/", b"foo/bar"),
		];

		for (path, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.pop();
			assert_eq!(path_mut.as_bytes(), expected)
		}
	}

	#[test]
	fn pop_following_authority() {
		let vectors: [(&[u8], &[u8]); _] = [(b"", b"/.."), (b"/", b"/"), (b"/..", b"/../..")];

		for (path, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path_following_authority(&mut path);
			path_mut.pop();
			assert_eq!(path_mut.as_bytes(), expected)
		}
	}

	#[test]
	fn normalized() {
		let vectors: [(&[u8], &[u8]); 9] = [
			(b"", b""),
			(b"a/b/c", b"a/b/c"),
			(b"a/..", b""),
			(b"a/b/..", b"a"),
			(b"a/b/../", b"a/"),
			(b"a/b/c/..", b"a/b"),
			(b"a/b/c/.", b"a/b/c"),
			(b"a/../..", b".."),
			(b"/a/../..", b"/"),
		];

		for (input, expected) in vectors {
			let mut path = PathBuf::new(input.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.normalize();
			assert_eq!(path_mut.as_bytes(), expected);
		}
	}
}
