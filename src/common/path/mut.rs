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
			pub fn push(&mut self, segment: &super::Segment) {
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

			/// Pop the last non-`..` segment of the path.
			///
			/// If the path is empty and relative, or ends in `..`, then a `..` segment
			/// will be added instead.
			///
			/// Returns `true` if the path has been modified, or `false` otherwise.
			pub fn pop(&mut self) -> bool {
				let is_empty = self.is_empty();

				if (is_empty && self.is_relative()) || self.last() == Some(super::Segment::PARENT) {
					self.push(super::Segment::PARENT);
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

			/// Push the given segment to this path using the `.` and `..` segments
			/// semantics.
			///
			/// Returns wether or not a special segment has been push and should be
			/// followed by an empty segment when doing reference resolution.
			#[inline]
			pub(crate) fn symbolic_push_inner(&mut self, segment: &super::Segment) -> bool {
				match segment.as_bytes() {
					CURRENT_SEGMENT => true,
					PARENT_SEGMENT => {
						self.pop();
						true
					}
					_ => {
						if !segment.is_empty() || !self.is_empty() {
							self.push(segment);
						}

						false
					}
				}
			}

			/// Push the given segment to this path using the `.` and `..` segments
			/// semantics.
			#[inline]
			pub fn symbolic_push(&mut self, segment: &super::Segment) {
				if self.symbolic_push_inner(segment) && !self.is_empty() {
					self.push(super::Segment::EMPTY)
				}
			}

			/// Append the given path to this path using the `.` and `..` segments semantics.
			///
			/// Note that this does not normalize the segments already in the path.
			/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
			/// `a/` because the semantics of `..` is applied on the last `.` in the path.
			#[inline]
			pub fn symbolic_append<'s, S: IntoIterator<Item = &'s super::Segment>>(
				&mut self,
				path: S,
			) {
				let mut open = false;
				for segment in path {
					open = self.symbolic_push_inner(segment);
				}

				if open && !self.is_empty() {
					self.push(super::Segment::EMPTY)
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
