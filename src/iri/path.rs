use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::iter::IntoIterator;
use pct_str::PctStr;
use crate::{parsing, IriRefBuf};
use super::{Error, Segment};

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
			data: &self.data,
			offset: 0
		}
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
			data: self.data,
			offset: 0
		}
	}
}

pub struct Segments<'a> {
	data: &'a [u8],
	offset: usize
}

impl<'a> Iterator for Segments<'a> {
	type Item = Segment<'a>;

	fn next(&mut self) -> Option<Segment<'a>> {
		let mut start = self.offset;
		let mut end = self.offset;

		loop {
			match parsing::get_char(self.data, end).unwrap() {
				Some(('/', 1)) => {
					if end == self.offset {
						start += 1;
						end += 1;
					} else {
						break
					}
				},
				Some((_, len)) => {
					end += len;
				},
				None => break
			}
		}

		self.offset = end;

		if end > start {
			Some(Segment {
				data: &self.data[start..end]
			})
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
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Path<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Path<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_pct_str() == *other
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

	/// Add a segment at the end of the path.
	pub fn push(&mut self, segment: Segment) {
		let segment = segment.as_ref();
		if segment.is_empty() {
			if self.buffer.path().as_str() == "/" {
				// This is the edge case!
				// We can't have the path starting with `//` without an explicit authority part.
				// So we make sure the authority fragment is showing with `://`.
				self.buffer.authority_mut().make_explicit();
			} else {
				// make sure it ends with a slash.
				self.open();
			}

			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
			self.buffer.p.path_len += 1;
		} else {
			self.open();
			// add the segment at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, segment);
			self.buffer.p.path_len += segment.len();
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
		}
	}

	pub fn symbolic_append<'s, P: IntoIterator<Item = Segment<'s>>>(&mut self, path: P) {
		for segment in path {
			match segment.as_str() {
				"." => self.open(),
				".." => self.pop(),
				_ => self.push(segment)
			}
		}
	}

	pub fn remove_dot_segments(&mut self) {
		let mut path_buffer = if self.is_absolute() {
			IriRefBuf::new("/").unwrap()
		} else {
			IriRefBuf::default()
		};

		path_buffer.path_mut().symbolic_append(self.as_path());
		if self.as_path().is_open() {
			path_buffer.path_mut().open();
		}

		let offset = self.buffer.p.path_offset();
		let end = offset + self.as_ref().len();
		self.buffer.replace(offset..end, path_buffer.path().as_ref());
		self.buffer.p.path_len = path_buffer.len();

		// Make the authority explicit if we need to.
		if self.buffer.data.len() >= offset + 2 && self.buffer.data[offset] == 0x2f && self.buffer.data[offset + 1] == 0x2f {
			self.buffer.authority_mut().make_explicit();
		}
	}
}

#[cfg(test)]
mod tests {
	use std::convert::TryInto;
	use crate::{Iri, IriBuf};

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
	fn push_empty_segment_edge_case() {
		let mut iri = IriBuf::new("scheme:/").unwrap();
		let mut path = iri.path_mut();

		path.push("".try_into().unwrap());

		assert_eq!(iri.as_str(), "scheme:////");
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
}
