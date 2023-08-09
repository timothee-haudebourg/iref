use std::ops::Deref;

use smallvec::SmallVec;

use crate::utils::{allocate_range, replace};

use super::{path::Segment, Path, PathBuf};

/// Stack size (in bytes) allocated for the `normalize` method to normalize a
/// `Path`. If it needs more space, it will allocate memory on the heap.
const NORMALIZE_IN_PLACE_BUFFER_LEN: usize = 512;

/// Mutable IRI path.
pub struct PathMut<'a> {
	/// Buffer storing the path.
	buffer: &'a mut Vec<u8>,

	/// Start offset (included).
	start: usize,

	/// End offset (excluded).
	end: usize,
}

impl<'a> Deref for PathMut<'a> {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		unsafe {
			Path::new_unchecked(std::str::from_utf8_unchecked(
				&self.buffer[self.start..self.end],
			))
		}
	}
}

impl<'a> PathMut<'a> {
	pub fn from_path(path: &'a mut PathBuf) -> Self {
		let buffer = unsafe {
			// Safe because `PathMut` preserves well formed paths.
			path.as_mut_vec()
		};
		let end = buffer.len();

		Self {
			buffer,
			start: 0,
			end,
		}
	}

	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() {
			self.start + 1
		} else {
			self.start
		}
	}

	/// Add a segment at the end of the path.
	///
	/// # Ambiguities
	///
	/// Adding a segment to an empty path may introduce ambiguities and several
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
	/// ## Segment containing a `:`
	///
	/// If the path does not follow a scheme and/or authority part, a `:` in
	/// the first segment may be confused with a scheme separator
	/// (e.g. `looks-like-a-scheme:rest`).
	/// To avoid such ambiguity, this function will add a `.` segment to the
	/// path, preserving its semantics (e.g. `./looks-like-a-scheme:rest`).
	pub fn push(&mut self, segment: &Segment) {
		let disambiguate = self.is_empty()
			&& (segment.is_empty() || (self.start == 0 && segment.looks_like_scheme()));

		if disambiguate {
			let start = self.first_segment_offset();

			let len = 2 + segment.len();
			allocate_range(&mut self.buffer, start..start, len);
			self.end += len;
			let offset = start + 2;
			self.buffer[start..offset].copy_from_slice(b"./");
			self.buffer[offset..self.end].copy_from_slice(segment.as_bytes());
		} else if self.is_empty() {
			replace(&mut self.buffer, self.end..self.end, segment.as_bytes());
			self.end += segment.len();
		} else {
			let len = 1 + segment.len();
			allocate_range(&mut self.buffer, self.end..self.end, len);
			let offset = self.end + 1;
			self.buffer[self.end..offset].copy_from_slice(b"/");
			self.end += len;
			self.buffer[offset..self.end].copy_from_slice(segment.as_bytes());
		}
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead.
	pub fn pop(&mut self) {
		let is_empty = self.is_empty();

		if is_empty || self.last() == Some(Segment::PARENT) {
			self.push(Segment::PARENT)
		} else if !is_empty {
			let start = self.first_segment_offset();
			let mut i = self.end - 1;

			while i > start && self.buffer[i] != b'/' {
				i -= 1
			}

			replace(&mut self.buffer, i..self.end, &[]);
			self.end = i;
		}
	}

	pub fn clear(&mut self) {
		let start = self.first_segment_offset();
		replace(&mut self.buffer, start..self.end, b"");
		self.end = start
	}

	/// Push the given segment to this path using the `.` and `..` segments semantics.
	#[inline]
	pub fn symbolic_push(&mut self, segment: &Segment) {
		match segment.as_bytes() {
			b"." => (),
			b".." => self.pop(),
			_ => self.push(segment),
		}
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	#[inline]
	pub fn symbolic_append<'s, P: IntoIterator<Item = &'s Segment>>(&mut self, path: P) {
		for segment in path {
			self.symbolic_push(segment)
		}
	}

	#[inline]
	pub fn normalize(&mut self) {
		let mut buffer: SmallVec<[u8; NORMALIZE_IN_PLACE_BUFFER_LEN]> = SmallVec::new();
		for segment in self.normalized_segments() {
			if !buffer.is_empty() {
				buffer.push(b'/')
			}

			buffer.extend_from_slice(segment.as_bytes())
		}

		let start = self.first_segment_offset();
		replace(&mut self.buffer, start..self.end, &buffer);
		self.end = start + buffer.len();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn push() {
		let vectors = [
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
			let mut path = PathBuf::new(path.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.push(segment);
			assert_eq!(path_mut.as_str(), expected)
		}
	}

	#[test]
	fn pop() {
		let vectors = [
			("", ".."),
			("/", "/.."),
			("/..", "/../.."),
			("foo", ""),
			("foo/bar", "foo"),
			("foo/bar/", "foo/bar"),
		];

		for (path, expected) in vectors {
			let mut path = PathBuf::new(path.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.pop();
			assert_eq!(path_mut.as_str(), expected)
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
			let mut path = PathBuf::new(input.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.normalize();
			assert_eq!(path_mut.as_str(), expected);
		}
	}
}
