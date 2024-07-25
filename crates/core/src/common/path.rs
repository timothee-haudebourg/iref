use pct_str::PctStr;
use smallvec::SmallVec;

use super::{parse, path_mut::PathMutImpl};

pub const CURRENT_SEGMENT: &[u8] = b".";

pub const PARENT_SEGMENT: &[u8] = b"..";

pub trait PathBufImpl: 'static + Default {
	type Borrowed: ?Sized + PathImpl<Owned = Self>;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>;

	fn as_bytes(&self) -> &[u8];

	fn as_path_mut(&mut self) -> PathMutImpl<Self::Borrowed> {
		PathMutImpl::from_path(self)
	}

	fn is_empty(&self) -> bool {
		self.as_bytes().is_empty() || self.as_bytes() == *b"/"
	}
}

pub trait SegmentImpl: 'static {
	const PARENT: &'static Self;

	const EMPTY: &'static Self;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	fn as_bytes(&self) -> &[u8];

	fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_bytes()) }
	}

	fn len(&self) -> usize {
		self.as_bytes().len()
	}

	fn is_empty(&self) -> bool {
		self.as_bytes().is_empty()
	}

	/// Checks if this segment looks like a scheme.
	///
	/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
	/// valid scheme, of `false` otherwise.
	fn looks_like_scheme(&self) -> bool {
		parse::looks_like_scheme(self.as_bytes())
	}
}

pub trait PathImpl: 'static {
	const EMPTY: &'static Self;

	const EMPTY_ABSOLUTE: &'static Self;

	type Segment: ?Sized + SegmentImpl;

	type Owned: PathBufImpl<Borrowed = Self>;

	fn as_bytes(&self) -> &[u8];

	fn to_path_buf(&self) -> Self::Owned;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	/// Returns the byte lenght of the path.
	fn len(&self) -> usize {
		self.as_bytes().len()
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path has no segments.
	/// The absolute path `/` is empty.
	#[inline(always)]
	fn is_empty(&self) -> bool {
		self.as_bytes().is_empty() || self.as_bytes() == *b"/"
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	#[inline(always)]
	fn is_absolute(&self) -> bool {
		self.as_bytes().starts_with(b"/")
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	#[inline(always)]
	fn is_relative(&self) -> bool {
		!self.is_absolute()
	}

	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() {
			1
		} else {
			0
		}
	}

	fn first(&self) -> Option<&Self::Segment> {
		if self.is_empty() {
			None
		} else {
			Some(unsafe { self.segment_at(self.first_segment_offset()).0 })
		}
	}

	fn last(&self) -> Option<&Self::Segment> {
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
	unsafe fn segment_at(&self, offset: usize) -> (&Self::Segment, usize) {
		let mut i = offset;

		let bytes = self.as_bytes();
		while i < bytes.len() && !matches!(bytes[i], b'/' | b'?' | b'#') {
			i += 1
		}

		(
			<Self::Segment as SegmentImpl>::new_unchecked(&bytes[offset..i]),
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
	unsafe fn next_segment_from(&self, offset: usize) -> Option<(&Self::Segment, usize)> {
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
	unsafe fn previous_segment_from(&self, offset: usize) -> Option<(&Self::Segment, usize)> {
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
	fn segments(&self) -> SegmentsImpl<Self> {
		if self.is_empty() {
			SegmentsImpl::Empty
		} else {
			SegmentsImpl::NonEmpty {
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
	fn normalized_segments(&self) -> NormalizedSegmentsImpl<Self> {
		NormalizedSegmentsImpl::new(self)
	}

	#[inline]
	fn normalized(&self) -> Self::Owned {
		let mut result: Self::Owned = if self.is_absolute() {
			Self::EMPTY_ABSOLUTE.to_path_buf()
		} else {
			Self::EMPTY.to_path_buf()
		};

		let mut open = false;
		for segment in self.segments() {
			open = result.as_path_mut().symbolic_push(segment)
		}

		if open && !result.is_empty() {
			result.as_path_mut().push(Self::Segment::EMPTY)
		}

		result
	}

	/// Returns the last segment of the path, if there is one, unless it is
	/// empty.
	///
	/// This does not consider the normalized version of the path, dot segments
	/// are preserved.
	#[inline]
	fn file_name(&self) -> Option<&Self::Segment> {
		self.segments().next_back().filter(|s| !s.is_empty())
	}

	/// Returns the directory path, which is the path without the file name.
	fn directory(&self) -> &Self {
		let bytes = self.as_bytes();
		if bytes.is_empty() {
			self
		} else {
			let mut i = bytes.len() - 1;

			while i > 0 && bytes[i] != b'/' {
				i -= 1
			}

			if i == 0 && bytes[i] != b'/' {
				Self::EMPTY
			} else {
				unsafe { Self::new_unchecked(&bytes[..=i]) }
			}
		}
	}

	/// Returns the path without its final segment, if there is one.
	#[inline]
	fn parent(&self) -> Option<&Self> {
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
				unsafe { Some(Self::new_unchecked(b"/./")) }
			} else {
				unsafe { Some(Self::new_unchecked(&bytes[..end])) }
			}
		}
	}

	/// Returns the path without its final segment, if there is one.
	#[inline]
	fn parent_or_empty(&self) -> &Self {
		self.parent().unwrap_or_else(|| {
			if self.is_absolute() {
				Self::EMPTY_ABSOLUTE
			} else {
				Self::EMPTY
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
	/// # use iref_core as iref;
	/// use iref::iri::{Path, PathBuf};
	///
	/// let prefix = Path::new("/foo/bar").unwrap();
	/// let path = Path::new("/foo/bar/baz").unwrap();
	/// let suffix: PathBuf = path.suffix(prefix).unwrap();
	///
	/// assert_eq!(suffix.as_str(), "baz");
	/// ```
	#[inline]
	fn suffix(&self, prefix: &Self) -> Option<Self::Owned> {
		if self.is_absolute() != prefix.is_absolute() {
			return None;
		}

		let mut buf: Self::Owned = Default::default();
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

	/// Checks if this path looks like a scheme.
	///
	/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
	/// valid scheme, of `false` otherwise.
	fn looks_like_scheme(&self) -> bool {
		parse::looks_like_scheme(self.as_bytes())
	}
}

pub enum SegmentsImpl<'a, P: ?Sized> {
	Empty,
	NonEmpty {
		path: &'a P,
		offset: usize,
		back_offset: usize,
	},
}

impl<'a, P: ?Sized + PathImpl> Iterator for SegmentsImpl<'a, P> {
	type Item = &'a P::Segment;

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

impl<'a, P: ?Sized + PathImpl> DoubleEndedIterator for SegmentsImpl<'a, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			SegmentsImpl::Empty => None,
			SegmentsImpl::NonEmpty {
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

pub struct NormalizedSegmentsImpl<'a, P: ?Sized + PathImpl>(
	smallvec::IntoIter<[&'a P::Segment; NORMALIZE_STACK_SIZE]>,
);

impl<'a, P: ?Sized + PathImpl> NormalizedSegmentsImpl<'a, P> {
	fn new(path: &'a P) -> NormalizedSegmentsImpl<P> {
		let relative = path.is_relative();
		let mut stack = SmallVec::<[&'a P::Segment; NORMALIZE_STACK_SIZE]>::new();

		for segment in path.segments() {
			match segment.as_bytes() {
				CURRENT_SEGMENT => (),
				PARENT_SEGMENT => {
					if stack
						.last()
						.map(|s| s.as_bytes() == PARENT_SEGMENT)
						.unwrap_or(relative)
					{
						stack.push(segment)
					} else {
						stack.pop();
					}
				}
				_ => stack.push(segment),
			}
		}

		NormalizedSegmentsImpl(stack.into_iter())
	}
}

impl<'a, P: ?Sized + PathImpl> Iterator for NormalizedSegmentsImpl<'a, P> {
	type Item = &'a P::Segment;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<&'a P::Segment> {
		self.0.next()
	}
}

impl<'a, P: ?Sized + PathImpl> DoubleEndedIterator for NormalizedSegmentsImpl<'a, P> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<'a, P: ?Sized + PathImpl> ExactSizeIterator for NormalizedSegmentsImpl<'a, P> {}
