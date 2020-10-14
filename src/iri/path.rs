use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::iter::IntoIterator;
use smallvec::SmallVec;
use pct_str::PctStr;
use crate::{parsing, IriRef, IriRefBuf, AsIriRef};
use super::{Error, Segment};

#[derive(Clone, Copy)]
pub struct Path<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Path<'a> {
	/// The inner data length (in bytes), without the trailing `/` is the path is open.
	#[inline]
	fn closed_len(&self) -> usize {
		if self.is_open() {
			self.data.len() - 1
		} else {
			self.data.len()
		}
	}

	#[inline]
	pub fn as_ref(&self) -> &[u8] {
		self.data
	}

	#[inline]
	pub fn into_ref(self) -> &'a [u8] {
		self.data
	}

	/// Get the underlying path slice as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.data)
		}
	}

	/// Convert this path into the underlying path slice.
	#[inline]
	pub fn into_str(self) -> &'a str {
		unsafe {
			std::str::from_utf8_unchecked(self.data)
		}
	}

	/// Get the underlying path slice as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

	/// Get the path slice as an IRI reference.
	#[inline]
	pub fn as_iri_ref(&self) -> IriRef {
		IriRef {
			p: parsing::ParsedIriRef {
				path_len: self.data.len(),
				..parsing::ParsedIriRef::default()
			},
			data: self.data
		}
	}

	/// Convert this path into an IRI reference.
	#[inline]
	pub fn into_iri_ref(self) -> IriRef<'a> {
		IriRef {
			p: parsing::ParsedIriRef {
				path_len: self.data.len(),
				..parsing::ParsedIriRef::default()
			},
			data: self.data
		}
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty() || self.data == &[0x2f]
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	#[inline]
	pub fn is_absolute(&self) -> bool {
		!self.data.is_empty() && self.data[0] == 0x2f
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	#[inline]
	pub fn is_relative(&self) -> bool {
		self.data.is_empty() || self.data[0] != 0x2f
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	#[inline]
	pub fn is_open(&self) -> bool {
		self.data.len() > 1 && self.data.last() == Some(&0x2f)
	}

	#[inline]
	pub fn is_closed(&self) -> bool {
		!self.is_open()
	}

	fn segment_at(&self, offset: usize) -> (Option<Segment<'a>>, usize) {
		let mut start = offset;
		let mut end = offset;

		loop {
			match parsing::get_char(self.data, end).unwrap() {
				Some(('/', 1)) => {
					if end == offset {
						start += 1;
						end += 1;
					} else {
						break
					}
				},
				Some((_, len)) => {
					end += len;
				},
				None => {
					if end == start {
						return (None, end)
					} else {
						break
					}
				}
			}
		}

		(Some(Segment {
			data: &self.data[start..end],
			open: self.data.get(end) == Some(&0x2f)
		}), end)
	}

	#[inline]
	pub fn first(&self) -> Option<Segment<'a>> {
		let (segment, _) = self.segment_at(0);
		segment
	}

	/// Return the path directory part.
	///
	/// This correspond to the path without everything after the right most `/`.
	#[inline]
	pub fn directory(&self) -> Path {
		if self.data.is_empty() {
			Path {
				data: &[]
			}
		} else {
			let mut last = self.data.len() - 1;

			loop {
				if self.data[last] == 0x2f {
					break;
				}

				if last == 0 {
					return Path {
						data: &[]
					}
				}

				last -= 1;
			}

			Path {
				data: &self.data[0..(last + 1)]
			}
		}
	}

	/// Produces an iterator over the segments of the IRI path.
	///
	/// Note that this is an IRI path, not an IRI reference path: no normalization occurs with
	/// `.` and `..` segments. This is done by the IRI reference resolution function.
	///
	/// Empty segments are preserved: the path `a//b` will raise the three segments `a`, `` and
	/// `b`.
	/// The absolute path `/` has no segments, but the path `/a/` has two segments, `a` and ``.
	#[inline]
	pub fn segments(&self) -> Segments {
		Segments::new(*self)
	}

	/// Iterate over the normalized segments of the path.
	///
	/// Remove the special dot segments `..` and `.` from the iteration using the usual path
	/// semantics for dot segments.
	/// This may be expensive for large paths since it will need to internally normalize the path
	/// first.
	#[inline]
	pub fn normalized_segments(&self) -> NormalizedSegments {
		NormalizedSegments::new(*self)
	}

	/// Consume the path reference and return an iterator over its normalized segments.
	#[inline]
	pub fn into_normalized_segments(self) -> NormalizedSegments<'a> {
		NormalizedSegments::new(self)
	}

	/// Returns the name of the final segment of the path, if there is one.
	///
	/// If the path is a normal file, this is the file name. If it's the path of a directory, this
	/// is the directory name.
	///
	/// This does not consider the normalized version of the path, dot segments are preserved.
	#[inline]
	pub fn file_name(&self) -> Option<&'a str> {
		match self.into_iter().next_back() {
			Some(s) => Some(s.into_str()),
			None => None
		}
	}

	/// Returns the path without its final component, if there is one.
	#[inline]
	pub fn parent(&self) -> Option<Path<'a>> {
		let mut i = self.closed_len();

		if self.is_empty() {
			None
		} else {
			i -= 1;

			loop {
				if self.data[i] == 0x2f {
					break
				} else {
					if i > 0 {
						i -= 1;
					} else {
						return None
					}
				}
			}

			Some(Path {
				data: &self.data[0..(i+1)]
			})
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
	/// use iref::{Path, PathBuf};
	///
	/// let prefix = Path::try_from("/foo/bar").unwrap();
	/// let path = Path::try_from("/foo/bar/baz").unwrap();
	/// let suffix: PathBuf = path.suffix(prefix).unwrap();
	///
	/// assert_eq!(suffix, "baz");
	/// ```
	#[inline]
	pub fn suffix(&self, prefix: Path) -> Option<PathBuf> {
		if self.is_absolute() != prefix.is_absolute() {
			return None
		}

		let mut buf = PathBuf::new();
		let mut self_it = self.normalized_segments();
		let mut prefix_it = prefix.normalized_segments();

		loop {
			match (self_it.next(), prefix_it.next()) {
				(Some(self_seg), Some(prefix_seg)) if self_seg.as_pct_str() == prefix_seg.as_pct_str() => (),
				(_, Some(_)) => return None,
				(Some(seg), None) => buf.as_path_mut().push(seg),
				(None, None) => break
			}
		}

		Some(buf)
	}
}

impl<'a> AsIriRef for Path<'a> {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		self.as_iri_ref()
	}
}

impl<'a> TryFrom<&'a str> for Path<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<Path<'a>, Error> {
		let path_len = parsing::parse_path(str.as_ref(), 0)?;
		if path_len < str.len() {
			Err(Error::InvalidPath)
		} else {
			Ok(Path {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> IntoIterator for Path<'a> {
	type Item = Segment<'a>;
	type IntoIter = Segments<'a>;

	#[inline]
	fn into_iter(self) -> Segments<'a> {
		Segments::new(self)
	}
}

#[derive(Clone)]
pub struct Segments<'a> {
	path: Path<'a>,
	offset: usize,
	offset_back: usize
}

impl<'a> Segments<'a> {
	fn new(path: Path<'a>) -> Segments<'a> {
		let offset_back = path.closed_len();
		Segments {
			path,
			offset: 0,
			offset_back
		}
	}
}

impl<'a> Iterator for Segments<'a> {
	type Item = Segment<'a>;

	#[inline]
	fn next(&mut self) -> Option<Segment<'a>> {
		if self.offset >= self.offset_back {
			None
		} else {
			let (segment, end) = self.path.segment_at(self.offset);
			self.offset = end;
			segment
		}
	}
}

impl<'a> DoubleEndedIterator for Segments<'a> {
	#[inline]
	fn next_back(&mut self) -> Option<Segment<'a>> {
		if self.offset >= self.offset_back {
			None
		} else {
			let mut i = self.offset_back - 1; // Note that `offset_back` cannot be 0 here, or we
			                                   // wouldn't be in this branch.

			loop {
				if i > 0 {
					if self.path.data[i] == 0x2f {
						break
					} else {
						i -= 1;
					}
				} else {
					break
				}
			}

			self.offset_back = i;
			let (segment, _) = self.path.segment_at(self.offset_back);
			segment
		}
	}
}

/// Stack size (in `Segment`) allocated for [`NormalizedSegments`] to normalize a `Path`.
/// If it needs more space, it will allocate memory on the heap.
const NORMALIZE_STACK_SIZE: usize = 16;

/// Stack size (in bytes) allocated for the `normalize` method to normalize a `Path`.
/// If it needs more space, it will allocate memory on the heap.
const REMOVE_DOTS_BUFFER_LEN: usize = 512;

pub struct NormalizedSegments<'a> {
	stack: SmallVec<[Segment<'a>; NORMALIZE_STACK_SIZE]>,
	i: usize
}

impl<'a> NormalizedSegments<'a> {
	fn new(path: Path<'a>) -> NormalizedSegments {
		let relative = path.is_relative();
		let mut stack: SmallVec<[Segment<'a>; NORMALIZE_STACK_SIZE]> = SmallVec::new();
		for segment in path.into_iter() {
			match segment.as_str() {
				"." => {
					if let Some(last_segment) = stack.last_mut().as_mut() {
						last_segment.open();
					}
				},
				".." => {
					if stack.pop().is_none() && relative {
						stack.push(segment)
					}
				},
				_ => stack.push(segment)
			}
		}

		NormalizedSegments {
			stack, i: 0
		}
	}
}

impl<'a> Iterator for NormalizedSegments<'a> {
	type Item = Segment<'a>;

	#[inline]
	fn next(&mut self) -> Option<Segment<'a>> {
		if self.i < self.stack.len() {
			let segment = self.stack[self.i];
			self.i += 1;
			Some(segment)
		} else {
			None
		}
	}
}

impl<'a> fmt::Display for Path<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Path<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Path<'a> {
	#[inline]
	fn eq(&self, other: &Path) -> bool {
		if self.is_absolute() == other.is_absolute() {
			let mut self_segments = self.normalized_segments();
			let mut other_segments = other.normalized_segments();

			loop {
				match (self_segments.next(), other_segments.next()) {
					(None, None) => return true,
					(Some(_), None) => return false,
					(None, Some(_)) => return false,
					(Some(a), Some(b)) => {
						if a != b {
							return false
						}
					}
				}
			}
		} else {
			false
		}
	}
}

impl<'a> Eq for Path<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Path<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Path::try_from(*other) {
			self == &other
		} else {
			false
		}
	}
}

impl<'a> PartialOrd for Path<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Path<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Path<'a> {
	#[inline]
	fn cmp(&self, other: &Path<'a>) -> Ordering {
		if self.is_absolute() == other.is_absolute() {
			let mut self_segments = self.normalized_segments();
			let mut other_segments = other.normalized_segments();

			loop {
				match (self_segments.next(), other_segments.next()) {
					(None, None) => return Ordering::Equal,
					(Some(_), None) => return Ordering::Greater,
					(None, Some(_)) => return Ordering::Less,
					(Some(a), Some(b)) => {
						match a.cmp(&b) {
							Ordering::Greater => return Ordering::Greater,
							Ordering::Less => return Ordering::Less,
							Ordering::Equal => ()
						}
					}
				}
			}
		} else if self.is_absolute() {
			Ordering::Greater
		} else {
			Ordering::Less
		}
	}
}

impl<'a> Hash for Path<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

pub struct PathMut<'a> {
	pub(crate) buffer: &'a mut IriRefBuf
}

impl<'a> PathMut<'a> {
	/// Get the inner path.
	#[inline]
	pub fn as_path(&self) -> Path {
		self.buffer.path()
	}

	/// Get the underlying byte slice.
	#[inline]
	pub fn as_ref(&self) -> &[u8] {
		let offset = self.buffer.p.path_offset();
		let len = self.buffer.path().as_ref().len();
		&self.buffer.data[offset..(offset+len)]
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.buffer.path().is_empty()
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	#[inline]
	pub fn is_absolute(&self) -> bool {
		self.buffer.path().is_absolute()
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	#[inline]
	pub fn is_relative(&self) -> bool {
		self.buffer.path().is_relative()
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	#[inline]
	pub fn is_open(&self) -> bool {
	 	self.buffer.path().is_open()
	}

	#[inline]
	pub fn is_closed(&self) -> bool {
		self.buffer.path().is_closed()
	}

	/// Make sure the last segment is followed by a `/`.
	///
	/// This has no effect if the path is empty.
	#[inline]
	pub fn open(&mut self) {
		if !self.is_empty() && self.is_closed() {
			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
			self.buffer.p.path_len += 1;
		}
	}

	/// Produces an iterator over the segments of the IRI path.
	///
	/// Note that this is an IRI path, not an IRI reference path: no normalization occurs with
	/// `.` and `..` segments. This is done by the IRI reference resolution function.
	///
	/// Empty segments are preserved: the path `a//b` will raise the three segments `a`, `` and
	/// `b`.
	/// The absolute path `/` has no segments, but the path `/a/` has two segments, `a` and ``.
	#[inline]
	pub fn segments(&self) -> Segments {
		self.buffer.path().into_iter()
	}

	#[inline]
	pub fn normalized_segments(&self) -> NormalizedSegments {
		self.buffer.path().into_normalized_segments()
	}

	pub(crate) fn disambiguate(&mut self) {
		if let Some(first) = self.as_path().first() {
			if (first.is_empty() && self.buffer.authority().is_none()) ||
			   (self.is_relative() && self.buffer.scheme().is_none() && self.buffer.authority().is_none() && first.as_ref().contains(&0x3a)) {
				// add `./` at the begining.
				let mut offset = self.buffer.p.path_offset();
				if self.is_absolute() {
					offset += 1;
				}
				self.buffer.replace(offset..offset, &[0x2e, 0x2f]);
				self.buffer.p.path_len += 2;
			}
		}
	}

	/// Add a segment at the end of the path.
	#[inline]
	pub fn push<'s>(&mut self, segment: Segment<'s>) {
		if segment.is_empty() {
			// if the whole IRI is of the form (1) `scheme:?query#fragment` or (2) `scheme:/?query#fragment`,
			// we must add a `./` before this segment to make sure that
			// (1) we don't ambiguously add a `/` at the begining making the path absolute.
			// (2) we don't make the path start with `//`, confusing it with the authority.
			if self.is_empty() && self.buffer.authority().is_none() {
				self.push(Segment::current())
			}

			// make sure it ends with a slash.
			self.open();

			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
			self.buffer.p.path_len += 1;
		} else {
			// if the whole IRI is of the form `?query#fragment`, and we push a segment containing a `:`,
			// it may be confused with the scheme.
			// We must add a `./` before the first segment to remove the ambiguity.
			if self.is_relative() && self.is_empty() && self.buffer.scheme().is_none() && self.buffer.authority().is_none() && segment.as_ref().contains(&0x3a) {
				self.push(Segment::current())
			}

			// make sure it ends with a slash.
			self.open();

			// add the segment at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, segment.as_ref());
			self.buffer.p.path_len += segment.len();
		}

		if segment.is_open() {
			self.open();
		}
	}

	#[inline]
	pub fn pop(&mut self) {
		if !self.is_empty() {
			let end = self.buffer.p.path_offset() + self.buffer.p.path_len;
			let mut start = end - 1;

			// We remove the terminating `/`.
			if self.is_open() {
				start -= 1;
			}

			// Find the last segment start position.
			while start > 0 && self.buffer.data[start] != 0x2f {
				start -= 1;
			}

			if start > 0 || self.buffer.data[start] == 0x2f {
				start += 1;
			}

			self.buffer.replace(start..end, &[]);
			self.buffer.p.path_len -= end - start;
		} else if self.is_relative() {
			self.push(Segment::parent());
		}
	}

	#[inline]
	pub fn clear(&mut self) {
		let mut offset = self.buffer.p.path_offset();
		let mut len = self.as_ref().len();

		if self.is_absolute() {
			offset += 1;
			len -= 1;
		}

		self.buffer.replace(offset..(offset+len), &[]);
		self.buffer.p.path_len = offset - self.buffer.p.path_offset();
	}

	#[inline]
	pub fn symbolic_append<'s, P: IntoIterator<Item = Segment<'s>>>(&mut self, path: P) {
		for segment in path {
			match segment.as_str() {
				"." => self.open(),
				".." => {
					self.pop();
					if segment.is_open() {
						self.open()
					}
				},
				_ => self.push(segment)
			}
		}
	}

	#[inline]
	pub fn normalize(&mut self) {
		let mut buffer: SmallVec<[u8; REMOVE_DOTS_BUFFER_LEN]> = SmallVec::new();
		buffer.extend_from_slice(self.as_ref());
		let old_path = Path { data: buffer.as_ref() };

		self.clear();

		for segment in old_path.normalized_segments() {
			self.push(segment);
		}
	}
}

impl<'a> AsIriRef for PathMut<'a> {
	#[inline]
	fn as_iri_ref(&self) -> IriRef {
		self.as_path().into_iri_ref()
	}
}

impl<'a> PartialEq<PathMut<'a>> for Path<'a> {
	#[inline]
	fn eq(&self, other: &PathMut<'a>) -> bool {
		*self == other.as_path()
	}
}

impl<'a> PartialEq<Path<'a>> for PathMut<'a> {
	#[inline]
	fn eq(&self, other: &Path<'a>) -> bool {
		self.as_path() == *other
	}
}

/// A path buffer, that can be manipulated independently of an IRI.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct PathBuf {
	/// We actually store the path as an IRI-reference.
	data: IriRefBuf
}

impl PathBuf {
	/// Create a new empty path.
	#[inline]
	pub fn new() -> PathBuf {
		PathBuf {
			data: IriRefBuf::default()
		}
	}

	/// Consume the path and return its internal buffer.
	#[inline]
	pub fn into_bytes(self) -> Vec<u8> {
		self.data.into_bytes()
	}

	/// Borrow the internal buffer storing this path.
	#[inline]
	pub fn as_ref(&self) -> &[u8] {
		self.data.path().into_ref()
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		self.data.path().into_str()
	}

	#[inline]
	pub fn as_path(&self) -> Path {
		self.data.path()
	}

	#[inline]
	pub fn as_path_mut(&mut self) -> PathMut {
		self.data.path_mut()
	}

	/// Borrow the path as an IRI reference.
	#[inline]
	pub fn as_iri_ref(&self) -> IriRef {
		self.data.as_iri_ref()
	}

	/// Convert the path into an IRI reference.
	#[inline]
	pub fn into_iri_ref(self) -> IriRefBuf {
		self.data
	}
}

impl<'a> From<Path<'a>> for PathBuf {
	#[inline]
	fn from(path: Path<'a>) -> PathBuf {
		let mut buf = PathBuf::new();
		buf.data.replace(0..0, path.as_ref());
		buf.data.p.path_len = path.as_ref().len();
		buf
	}
}

impl<'a> From<NormalizedSegments<'a>> for PathBuf {
	#[inline]
	fn from(segments: NormalizedSegments<'a>) -> PathBuf {
		let mut buf = PathBuf::new();
		let mut path = buf.as_path_mut();
		for seg in segments {
			path.push(seg)
		}

		buf
	}
}

impl fmt::Display for PathBuf {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for PathBuf {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> PartialEq<Path<'a>> for PathBuf {
	#[inline]
	fn eq(&self, other: &Path<'a>) -> bool {
		self.data.path() == *other
	}
}

impl<'a> PartialEq<PathMut<'a>> for PathBuf {
	#[inline]
	fn eq(&self, other: &PathMut<'a>) -> bool {
		self.data.path() == *other
	}
}

impl<'a> PartialEq<&'a str> for PathBuf {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.data.path() == *other
	}
}

#[cfg(test)]
mod tests {
	use std::convert::{TryInto, TryFrom};
	use crate::{Iri, IriBuf, IriRefBuf, Path, PathBuf};

	#[test]
	fn empty() {
		let iri = Iri::new("scheme:").unwrap();
		let path = iri.path();

		assert!(path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.is_closed());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn empty_absolute() {
		let iri = Iri::new("scheme:/").unwrap();
		let path = iri.path();

		assert!(path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_closed());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn non_empty() {
		let iri = Iri::new("scheme:a/b").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.is_closed());

		let mut segments = path.segments();
		assert!(segments.next().unwrap() == "a");
		assert!(segments.next().unwrap() == "b");
		assert!(segments.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let iri = Iri::new("scheme:/foo/bar").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_closed());

		let mut segments = path.segments();
		assert!(segments.next().unwrap() == "foo");
		assert!(segments.next().unwrap() == "bar");
		assert!(segments.next().is_none());
	}

	#[test]
	fn is_open() {
		let iri = Iri::new("scheme:/red/green/blue/").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_open());

		let mut segments = path.segments();
		assert!(segments.next().unwrap() == "red");
		assert!(segments.next().unwrap() == "green");
		assert!(segments.next().unwrap() == "blue");
		assert!(segments.next().is_none());
	}

	#[test]
	fn push() {
		let mut iri = IriBuf::new("scheme:foo").unwrap();
		let mut path = iri.path_mut();

		path.push("bar".try_into().unwrap());

		assert_eq!(iri.as_str(), "scheme:foo/bar");
	}

	#[test]
	fn push_empty_segment() {
		let mut iri = IriBuf::new("scheme:foo/bar").unwrap();
		let mut path = iri.path_mut();

		path.push("".try_into().unwrap());

		assert_eq!(iri.as_str(), "scheme:foo/bar//");
	}

	#[test]
	fn push_empty_segment_edge_case1() {
		let mut iri = IriBuf::new("scheme:").unwrap();
		let mut path = iri.path_mut();

		path.push("".try_into().unwrap());

		assert_eq!(iri, "scheme:.//");
	}

	#[test]
	fn push_empty_segment_edge_case2() {
		let mut iri = IriBuf::new("scheme:/").unwrap();
		let mut path = iri.path_mut();

		path.push("".try_into().unwrap());

		assert_eq!(iri, "scheme:/.//");
	}

	#[test]
	fn push_scheme_edge_case() {
		let mut iri_ref = IriRefBuf::new("").unwrap();
		let mut path = iri_ref.path_mut();

		path.push("a:b".try_into().unwrap());

		assert_eq!(iri_ref.as_str(), "./a:b");
	}

	#[test]
	fn pop() {
		let mut iri = IriBuf::new("scheme:foo/bar").unwrap();
		let mut path = iri.path_mut();

		path.pop();

		assert_eq!(iri.as_str(), "scheme:foo/");
	}

	#[test]
	fn pop_open() {
		let mut iri = IriBuf::new("scheme:foo/bar/").unwrap();
		let mut path = iri.path_mut();

		path.pop();

		assert_eq!(iri.as_str(), "scheme:foo/");
	}

	#[test]
	fn pop_open_empty_segment() {
		let mut iri = IriBuf::new("scheme:foo//").unwrap();
		let mut path = iri.path_mut();

		path.pop();

		assert_eq!(iri.as_str(), "scheme:foo/");
	}

	#[test]
	fn pop_open_empty_segment_edge_case() {
		let mut iri = IriBuf::new("scheme:////").unwrap();
		let mut path = iri.path_mut();

		path.pop();

		assert_eq!(iri.as_str(), "scheme:///");
	}

	#[test]
	fn open() {
		let mut iri = IriBuf::new("scheme:/a").unwrap();
		let mut path = iri.path_mut();

		path.open();

		assert_eq!(iri.as_str(), "scheme:/a/");
	}

	#[test]
	fn compare() {
		assert_eq!(Path::try_from("a/b/c").unwrap(), "a/b/c");
		assert_eq!(Path::try_from("a/b/c/").unwrap(), "a/b/c/.");
		assert_eq!(Path::try_from("a/b/c/").unwrap(), "a/b/c/./");
		assert_eq!(Path::try_from("a/b/c").unwrap(), "a/b/../b/c");
		assert_eq!(Path::try_from("a/b/c/..").unwrap(), "a/b/");
		assert_eq!(Path::try_from("a/..").unwrap(), "");
		assert_eq!(Path::try_from("/a/..").unwrap(), "/");
		assert_eq!(Path::try_from("a/../..").unwrap(), "..");
		assert_eq!(Path::try_from("/a/../..").unwrap(), "/..");

		assert_ne!(Path::try_from("a/b/c").unwrap(), "a/b/c/");
		assert_ne!(Path::try_from("a/b/c").unwrap(), "a/b/c/.");
		assert_ne!(Path::try_from("a/b/c/..").unwrap(), "a/b");
	}

	#[test]
	fn segments() {
		let path = Path::try_from("//a/b/foo//bar/").unwrap();
		let mut segments = path.into_iter();

		assert_eq!(segments.next().unwrap(), "");
		assert_eq!(segments.next().unwrap(), "a");
		assert_eq!(segments.next().unwrap(), "b");
		assert_eq!(segments.next().unwrap(), "foo");
		assert_eq!(segments.next().unwrap(), "");
		assert_eq!(segments.next().unwrap(), "bar");
		assert_eq!(segments.next(), None);
	}

	#[test]
	fn empty_segments() {
		let path = Path::try_from("").unwrap();
		let mut segments = path.into_iter();

		assert_eq!(segments.next(), None);
	}

	#[test]
	fn reverse_segments() {
		let path = Path::try_from("//a/b/foo//bar/").unwrap();
		let mut segments = path.into_iter();

		assert_eq!(segments.next_back().unwrap(), "bar");
		assert_eq!(segments.next_back().unwrap(), "");
		assert_eq!(segments.next_back().unwrap(), "foo");
		assert_eq!(segments.next_back().unwrap(), "b");
		assert_eq!(segments.next_back().unwrap(), "a");
		assert_eq!(segments.next_back().unwrap(), "");
		assert_eq!(segments.next_back(), None);
	}

	#[test]
	fn empty_reverse_segments() {
		let path = Path::try_from("").unwrap();
		let mut segments = path.into_iter();

		assert_eq!(segments.next_back(), None);
	}

	#[test]
	fn double_ended_segments() {
		let path = Path::try_from("//a/b/foo//bar/").unwrap();
		let mut segments = path.into_iter();

		assert_eq!(segments.next_back().unwrap(), "bar");
		assert_eq!(segments.next().unwrap(), "");
		assert_eq!(segments.next_back().unwrap(), "");
		assert_eq!(segments.next().unwrap(), "a");
		assert_eq!(segments.next_back().unwrap(), "foo");
		assert_eq!(segments.next().unwrap(), "b");
		assert_eq!(segments.next_back(), None);
		assert_eq!(segments.next(), None);
	}

	#[test]
	fn file_name() {
		let path = Path::try_from("//a/b/foo//bar/").unwrap();
		assert_eq!(path.file_name().unwrap(), "bar");
	}

	#[test]
	fn parent1() {
		let path = Path::try_from("//a/b/foo//bar/").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//a/b/foo//").unwrap());
	}

	#[test]
	fn parent1_closed() {
		let path = Path::try_from("//a/b/foo//bar").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//a/b/foo//").unwrap());
	}

	#[test]
	fn parent2() {
		let path = Path::try_from("//a/b/foo//").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//a/b/foo/").unwrap());
	}

	#[test]
	fn parent3() {
		let path = Path::try_from("//a/b/foo/").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//a/b/").unwrap());
	}

	#[test]
	fn parent4() {
		let path = Path::try_from("//a/b/").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//a/").unwrap());
	}

	#[test]
	fn parent5() {
		let path = Path::try_from("//a/").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("//").unwrap());
	}

	#[test]
	fn parent6() {
		let path = Path::try_from("//").unwrap();
		assert_eq!(path.parent().unwrap(), Path::try_from("/").unwrap());
	}

	#[test]
	fn parent7() {
		let path = Path::try_from("/").unwrap();
		assert_eq!(path.parent(), None);
	}

	#[test]
	fn parent_empty() {
		let path = Path::try_from("").unwrap();
		assert_eq!(path.parent(), None);
	}

	#[test]
	fn suffix_simple() {
		let prefix = Path::try_from("/foo/bar").unwrap();
		let path = Path::try_from("/foo/bar/baz").unwrap();
		let suffix: PathBuf = path.suffix(prefix).unwrap();
		assert_eq!(suffix, "baz");
	}

	#[test]
	fn suffix_not() {
		let prefix = Path::try_from("/foo/bar").unwrap();
		let path = Path::try_from("/a/b/baz").unwrap();
		assert!(path.suffix(prefix).is_none());
	}
}
