use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::iter::IntoIterator;
use std::ops::Deref;
use smallvec::SmallVec;
use pct_str::PctStr;
use crate::{parsing, IriRefBuf};
use super::{Error, Segment};

#[derive(Clone, Copy)]
pub struct Path<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Path<'a> {
	pub fn as_ref(&self) -> &[u8] {
		self.data
	}

	/// Get the underlying path slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

	/// Get the underlying path slice as a percent-encoded string slice.
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty() || self.data == &[0x2f]
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	pub fn is_absolute(&self) -> bool {
		!self.data.is_empty() && self.data[0] == 0x2f
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	pub fn is_relative(&self) -> bool {
		self.data.is_empty() || self.data[0] != 0x2f
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	pub fn is_open(&self) -> bool {
		self.data.len() > 1 && self.data.last() == Some(&0x2f)
	}

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

	pub fn first(&self) -> Option<Segment<'a>> {
		let (segment, _) = self.segment_at(0);
		segment
	}

	/// Return the path directory part.
	///
	/// This correspond to the path without everything after the right most `/`.
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
	pub fn segments(&self) -> Segments {
		Segments {
			path: *self,
			offset: 0
		}
	}

	pub fn normalized_segments(&self) -> NormalizedSegments {
		NormalizedSegments::new(*self)
	}

	pub fn into_normalized_segments(self) -> NormalizedSegments<'a> {
		NormalizedSegments::new(self)
	}
}

impl<'a> TryFrom<&'a str> for Path<'a> {
	type Error = Error;

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

	fn into_iter(self) -> Segments<'a> {
		Segments {
			path: self,
			offset: 0
		}
	}
}

#[derive(Clone)]
pub struct Segments<'a> {
	path: Path<'a>,
	offset: usize
}

impl<'a> Iterator for Segments<'a> {
	type Item = Segment<'a>;

	fn next(&mut self) -> Option<Segment<'a>> {
		let (segment, end) = self.path.segment_at(self.offset);
		self.offset = end;
		segment
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
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Path<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Path<'a> {
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
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Path::try_from(*other) {
			self == &other
		} else {
			false
		}
	}
}

impl<'a> PartialOrd for Path<'a> {
	fn partial_cmp(&self, other: &Path<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Path<'a> {
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
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

pub struct PathMut<'a> {
	pub(crate) buffer: &'a mut IriRefBuf
}

impl<'a> PathMut<'a> {
	pub fn as_path(&self) -> Path {
		self.buffer.path()
	}

	pub fn as_ref(&self) -> &[u8] {
		let offset = self.buffer.p.path_offset();
		let len = self.buffer.path().as_ref().len();
		&self.buffer.data[offset..(offset+len)]
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	pub fn is_empty(&self) -> bool {
		self.buffer.path().is_empty()
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	pub fn is_absolute(&self) -> bool {
		self.buffer.path().is_absolute()
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	pub fn is_relative(&self) -> bool {
		self.buffer.path().is_relative()
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	pub fn is_open(&self) -> bool {
	 	self.buffer.path().is_open()
	}

	pub fn is_closed(&self) -> bool {
		self.buffer.path().is_closed()
	}

	/// Make sure the last segment is followed by a `/`.
	///
	/// This has no effect if the path is empty.
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
	pub fn segments(&self) -> Segments {
		self.buffer.path().into_iter()
	}

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

#[cfg(test)]
mod tests {
	use std::convert::{TryInto, TryFrom};
	use crate::{Iri, IriBuf, IriRefBuf, Path};

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
}
