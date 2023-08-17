use std::{marker::PhantomData, ops::Deref};

use smallvec::SmallVec;

use crate::utils::{allocate_range, replace};

use super::{
	parse,
	path::{PathBufImpl, PathImpl, SegmentImpl, CURRENT_SEGMENT, PARENT_SEGMENT},
};

/// Stack size (in bytes) allocated for the `normalize` method to normalize a
/// `Path`. If it needs more space, it will allocate memory on the heap.
const NORMALIZE_IN_PLACE_BUFFER_LEN: usize = 512;

/// Mutable URI/IRI path.
pub struct PathMutImpl<'a, P: ?Sized> {
	/// Buffer storing the path.
	buffer: &'a mut Vec<u8>,

	/// Start offset (included).
	start: usize,

	/// End offset (excluded).
	end: usize,

	/// Determines if the path follows an authority part,
	/// in which case some disambiguation rules applies.
	follows_authority: bool,

	p: PhantomData<P>,
}

impl<'a, P: ?Sized + PathImpl> Deref for PathMutImpl<'a, P> {
	type Target = P;

	fn deref(&self) -> &Self::Target {
		unsafe { P::new_unchecked(&self.buffer[self.start..self.end]) }
	}
}

impl<'a, P: ?Sized + PathImpl> PathMutImpl<'a, P> {
	pub unsafe fn new(buffer: &'a mut Vec<u8>, start: usize, end: usize) -> Self {
		let follows_authority = parse::find_authority(&buffer[..start], 0).is_ok();

		Self {
			buffer,
			start,
			end,
			follows_authority,
			p: PhantomData,
		}
	}

	pub fn from_path(path: &'a mut P::Owned) -> Self {
		let buffer = unsafe {
			// Safe because `PathMut` preserves well formed paths.
			path.as_mut_vec()
		};
		let end = buffer.len();

		Self {
			buffer,
			start: 0,
			end,
			follows_authority: true,
			p: PhantomData,
		}
	}

	fn first_segment_offset(&self) -> usize {
		if self.is_absolute() {
			self.start + 1
		} else {
			self.start
		}
	}

	pub fn push(&mut self, segment: &P::Segment) {
		// Disambiguate if the path is empty and one of the following is true:
		// - `segment` looks like a scheme and path is a the start.
		// - `segment` is empty, path is absolute and following an authority.
		// - `segment` is empty, path is relative.
		let disambiguate = self.is_empty()
			&& ((self.start == 0 && segment.looks_like_scheme()) || segment.is_empty());

		if disambiguate {
			let start = self.first_segment_offset();
			let len = 2 + segment.len();
			allocate_range(self.buffer, start..start, len);
			self.end += len;
			let offset = start + 2;
			self.buffer[start..offset].copy_from_slice(b"./");
			self.buffer[offset..self.end].copy_from_slice(segment.as_bytes());
		} else if self.is_empty() {
			replace(self.buffer, self.end..self.end, segment.as_bytes());
			self.end += segment.len();
		} else {
			let bytes = self.as_bytes();
			let mut start_offset = 0usize;
			if (self.follows_authority || bytes.len() > 3) && bytes.ends_with(b"/./") {
				// we can remove the `./` here.
				start_offset = 2;
			};

			let start = self.end - start_offset;
			let len = 1 + segment.len();
			allocate_range(self.buffer, start..self.end, len);

			self.buffer[start] = b'/';
			self.end += len - start_offset;
			let segment_offset = start + 1;
			self.buffer[segment_offset..self.end].copy_from_slice(segment.as_bytes());
		}
	}

	/// Pop the last non-`..` segment of the path.
	///
	/// If the path is empty or ends in `..`, then a `..` segment
	/// will be added instead.
	pub fn pop(&mut self) {
		let is_empty = self.is_empty();

		if is_empty || self.last().map(SegmentImpl::as_bytes) == Some(PARENT_SEGMENT) {
			self.push(<P::Segment as SegmentImpl>::PARENT)
		} else if !is_empty {
			let start = self.first_segment_offset();
			let mut i = self.end - 1;

			while i > start && self.buffer[i] != b'/' {
				i -= 1
			}

			replace(self.buffer, i..self.end, &[]);
			self.end = i;
		}
	}

	pub fn clear(&mut self) {
		let start = self.first_segment_offset();
		replace(self.buffer, start..self.end, b"");
		self.end = start
	}

	/// Push the given segment to this path using the `.` and `..` segments
	/// semantics.
	///
	/// Returns wether or not a special segment has been push and should be
	/// followed by an empty segment when doing reference resolution.
	#[inline]
	pub fn symbolic_push(&mut self, segment: &P::Segment, is_open: bool) -> bool {
		match segment.as_bytes() {
			CURRENT_SEGMENT => true,
			PARENT_SEGMENT => {
				self.pop();
				true
			}
			_ => {
				if !segment.is_empty() || !is_open || !self.is_empty() {
					self.push(segment);
				}

				false
			}
		}
	}

	/// Append the given path to this path using the `.` and `..` segments semantics.
	///
	/// Note that this does not normalize the segments already in the path.
	/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
	/// `a/` because the semantics of `..` is applied on the last `.` in the path.
	#[inline]
	pub fn symbolic_append<'s, S: IntoIterator<Item = &'s P::Segment>>(&mut self, path: S) {
		let mut open = false;
		for segment in path {
			open = self.symbolic_push(segment, open);
		}

		if open && !self.is_empty() {
			self.push(<P::Segment as SegmentImpl>::EMPTY)
		}
	}

	#[inline]
	pub fn normalize(&mut self) {
		let mut buffer: SmallVec<[u8; NORMALIZE_IN_PLACE_BUFFER_LEN]> = SmallVec::new();
		for (i, segment) in self.normalized_segments().enumerate() {
			if i > 0 {
				buffer.push(b'/')
			}

			buffer.extend_from_slice(segment.as_bytes())
		}

		let start = self.first_segment_offset();
		replace(self.buffer, start..self.end, &buffer);
		self.end = start + buffer.len();
	}
}
