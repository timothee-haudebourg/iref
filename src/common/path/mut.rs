macro_rules! path_mut_impl {
	($name:literal) => {
		const CURRENT_SEGMENT: &[u8] = b".";

		const PARENT_SEGMENT: &[u8] = b"..";

		/// Stack size (in bytes) allocated for the `normalize` method to normalize a
		/// `Path`. If it needs more space, it will allocate memory on the heap.
		const NORMALIZE_IN_PLACE_BUFFER_LEN: usize = 512;

		/// Mutable
		#[doc = $name]
		/// path.
		pub struct PathMut<'a> {
			/// Buffer storing the path.
			buffer: &'a mut Vec<u8>,

			/// Start offset (included).
			start: usize,

			/// End offset (excluded).
			end: usize,

			/// Determines if the path follows an authority part,
			/// in which case some disambiguation rules applies.
			follows_authority: bool,
		}

		impl<'a> core::ops::Deref for PathMut<'a> {
			type Target = super::Path;

			fn deref(&self) -> &Self::Target {
				unsafe { super::Path::new_unchecked_from_bytes(&self.buffer[self.start..self.end]) }
			}
		}

		impl<'a> PathMut<'a> {
			/// Creates a new mutable path reference.
			///
			/// # Safety
			///
			/// The buffer content between in the range `start..end` must be a valid
			/// IRI path.
			pub unsafe fn new(buffer: &'a mut Vec<u8>, start: usize, end: usize) -> Self {
				let follows_authority =
					crate::common::parse::find_authority(&buffer[..start], 0).is_ok();

				Self {
					buffer,
					start,
					end,
					follows_authority,
				}
			}

			pub fn from_path(path: &'a mut super::PathBuf) -> Self {
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
			/// Same as [`Self::push`] but does not interpret the `.` and `..`
			/// segments. They will be added literaly to the path.
			pub fn lazy_push(&mut self, segment: &super::Segment) {
				// Disambiguate if the path is empty and one of the following is true:
				// - `segment` looks like a scheme and path is a the start.
				// - `segment` is empty, path is absolute and following an authority.
				// - `segment` is empty, path is relative.
				let disambiguate = self.is_empty()
					&& ((self.start == 0 && segment.looks_like_scheme()) || segment.is_empty());

				if disambiguate {
					let start = self.first_segment_offset();
					let len = 2 + segment.len();
					crate::utils::allocate_range(self.buffer, start..start, len);
					self.end += len;
					let offset = start + 2;
					self.buffer[start..offset].copy_from_slice(b"./");
					self.buffer[offset..self.end].copy_from_slice(segment.as_bytes());
				} else if self.is_empty() {
					crate::utils::replace(self.buffer, self.end..self.end, segment.as_bytes());
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
					crate::utils::allocate_range(self.buffer, start..self.end, len);

					self.buffer[start] = b'/';
					self.end += len - start_offset;
					let segment_offset = start + 1;
					self.buffer[segment_offset..self.end].copy_from_slice(segment.as_bytes());
				}
			}

			/// Adds a segment at the end of the path.
			///
			/// Same as [`Self::lazy_push`] but accepts a `&str` instead of a
			/// [`&Segment`](super::Segment). Returns an error if the input
			/// string is not a valid path segment.
			pub fn try_lazy_push<'s>(
				&mut self,
				segment: &'s str,
			) -> Result<(), super::InvalidSegment<&'s str>> {
				self.lazy_push(segment.try_into()?);
				Ok(())
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
			#[inline]
			pub fn push(&mut self, segment: &super::Segment) {
				if self.push_inner(segment) && !self.is_empty() {
					self.lazy_push(super::Segment::EMPTY)
				}
			}

			/// Pushes the given segment to this path using the `.` and `..` segments
			/// semantics.
			///
			/// Same as [`Self::push`] but accepts a `&str` instead of
			/// a [`&Segment`](super::Segment). Returns an error if the input
			/// string is not a valid path segment.
			#[inline]
			pub fn try_push<'s>(
				&mut self,
				segment: &'s str,
			) -> Result<(), super::InvalidSegment<&'s str>> {
				self.push(segment.try_into()?);
				Ok(())
			}

			/// Append the given path to this path using the `.` and `..` segments semantics.
			///
			/// Note that this does not normalize the segments already in the path.
			/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
			/// `a/` because the semantics of `..` is applied on the last `.` in the path.
			#[inline]
			pub fn append<'s, S: IntoIterator<Item = &'s super::Segment>>(&mut self, path: S) {
				let mut open = false;
				for segment in path {
					open = self.push_inner(segment);
				}

				if open && !self.is_empty() {
					self.lazy_push(super::Segment::EMPTY)
				}
			}

			/// Append the given path to this path using the `.` and `..` segments semantics.
			///
			/// Note that this does not normalize the segments already in the path.
			/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
			/// `a/` because the semantics of `..` is applied on the last `.` in the path.
			///
			/// Same as [`Self::append`], but accepts `&str` instead of
			/// [`&Segment`](super::Segment). Returns an error if one item is
			/// not a valid segment.
			#[inline]
			pub fn try_append<'s, S: IntoIterator<Item = &'s str>>(
				&mut self,
				path: S,
			) -> Result<(), super::InvalidSegment<&'s str>> {
				let mut open = false;
				for segment in path {
					open = self.push_inner(segment.try_into()?);
				}

				if open && !self.is_empty() {
					self.lazy_push(super::Segment::EMPTY)
				}

				Ok(())
			}

			/// Joins this path to the given path.
			///
			/// If the input path is absolute, this is equivalent to
			/// [`Self::replace`]. If the input path is relative, this is
			/// equivalent to [`Self::append`].
			pub fn join(&mut self, path: &super::Path) {
				if path.is_absolute() {
					self.replace(path);
				} else {
					self.append(path);
				}
			}

			/// Pop the last non-`..` segment of the path.
			///
			/// If the path is empty and relative, or ends in `..`, then a `..` segment
			/// will be added instead.
			///
			/// Returns `true` if the path has been modified, or `false` otherwise.
			pub fn pop(&mut self) -> bool {
				let is_empty = self.is_empty();

				if (is_empty && self.is_relative()) || self.last() == Some(super::Segment::PARENT) {
					self.lazy_push(super::Segment::PARENT);
					true
				} else if !is_empty {
					let start = self.first_segment_offset();
					let mut i = self.end - 1;

					while i > start && self.buffer[i] != b'/' {
						i -= 1
					}

					crate::utils::replace(self.buffer, i..self.end, &[]);
					self.end = i;
					true
				} else {
					false
				}
			}

			pub fn clear(&mut self) {
				let start = self.first_segment_offset();
				crate::utils::replace(self.buffer, start..self.end, b"");
				self.end = start
			}

			pub fn replace(&mut self, path: &super::Path) {
				let range = self.start..self.end;

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
					self.buffer[actual_start..(actual_start + path.len())]
						.copy_from_slice(path.as_bytes());
					self.end = self.start + path.len() + 2;
				} else if has_authority && path.is_relative() {
					// VALIDITY: When an authority is present, the path must be
					//           absolute.
					let start = range.start;
					let actual_start = start + 1;
					crate::utils::allocate_range(self.buffer, range, path.len() + 1);
					self.buffer[start] = b'/';
					self.buffer[actual_start..(actual_start + path.len())]
						.copy_from_slice(path.as_bytes());
					self.end = self.start + path.len() + 1;
				} else if range.start == 0 && path.looks_like_scheme() {
					// AMBIGUITY: The URI `old/path` would become `new:path`, but `new`
					//            is not the scheme.
					// SOLUTION:  We change `new:path` to `./new:path`.
					let start = range.start;
					let actual_start = start + 2;
					crate::utils::allocate_range(self.buffer, range, path.len() + 2);
					self.buffer[start..actual_start].copy_from_slice(b"./");
					self.buffer[actual_start..(actual_start + path.len())]
						.copy_from_slice(path.as_bytes());
					self.end = self.start + path.len() + 2;
				} else {
					crate::utils::replace(self.buffer, range, path.as_bytes());
					self.end = self.start + path.len();
				}
			}

			#[inline]
			pub fn normalize(&mut self) {
				let mut buffer: smallvec::SmallVec<[u8; NORMALIZE_IN_PLACE_BUFFER_LEN]> =
					smallvec::SmallVec::new();

				for (i, segment) in self.normalized_segments().enumerate() {
					if i > 0 {
						buffer.push(b'/')
					}

					buffer.extend_from_slice(segment.as_bytes())
				}

				let start = self.first_segment_offset();
				crate::utils::replace(self.buffer, start..self.end, &buffer);
				self.end = start + buffer.len();
			}
		}
	};
}

pub(crate) use path_mut_impl;
