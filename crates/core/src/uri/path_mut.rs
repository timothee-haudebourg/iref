use crate::common::path_mut::PathMutImpl;
use core::ops::Deref;

use super::{path::Segment, Path, PathBuf};

/// Mutable IRI path.
pub struct PathMut<'a>(PathMutImpl<'a, Path>);

impl<'a> Deref for PathMut<'a> {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl<'a> PathMut<'a> {
	pub(crate) fn from_impl(i: PathMutImpl<'a, Path>) -> Self {
		Self(i)
	}

	pub fn from_path(path: &'a mut PathBuf) -> Self {
		Self(PathMutImpl::from_path(path))
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
		self.0.push(segment)
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead.
	pub fn pop(&mut self) {
		self.0.pop();
	}

	pub fn clear(&mut self) {
		self.0.clear()
	}

	/// Push the given segment to this path using the `.` and `..` segments semantics.
	#[inline]
	pub fn symbolic_push(&mut self, segment: &Segment) {
		if self.0.symbolic_push(segment) && !self.0.is_empty() {
			self.0.push(Segment::EMPTY)
		}
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	#[inline]
	pub fn symbolic_append<'s, P: IntoIterator<Item = &'s Segment>>(&mut self, path: P) {
		self.0.symbolic_append(path)
	}

	#[inline]
	pub fn normalize(&mut self) {
		self.0.normalize()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn push() {
		let vectors: [(&[u8], &[u8], &[u8]); 12] = [
			(b"", b"foo", b"foo"),
			(b"/", b"foo", b"/foo"),
			(b"", b"", b"./"),
			(b"/", b"", b"/./"),
			(b"foo", b"bar", b"foo/bar"),
			(b"/foo", b"bar", b"/foo/bar"),
			(b"foo", b"", b"foo/"),
			(b"foo/bar", b"", b"foo/bar/"),
			(b"foo/", b"", b"foo//"),
			(b"a/b/c", b"d", b"a/b/c/d"),
			(b"/a/b/c", b"d", b"/a/b/c/d"),
			(b"a/b/c/", b"d", b"a/b/c//d"),
		];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.push(segment);
			assert_eq!(path_mut.as_bytes(), expected)
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
