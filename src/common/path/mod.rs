mod segment;
pub(crate) use segment::*;

mod r#mut;
pub(crate) use r#mut::*;

macro_rules! path_impl {
	($name:literal) => {
		const CURRENT_SEGMENT: &[u8] = b".";

		const PARENT_SEGMENT: &[u8] = b"..";

		#[doc = $name]
		/// path.
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::grammar::Path)]
		#[newtype(ord([u8], &[u8], Vec<u8>, str, &str, String), owned(PathBuf, derive(Default)))]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct Path(str);

		impl Default for &Path {
			fn default() -> Self {
				Path::EMPTY
			}
		}

		impl Path {
			pub const EMPTY: &'static Self = unsafe { Self::new_unchecked("") };

			/// The empty absolute path `/`.
			pub const EMPTY_ABSOLUTE: &'static Self = unsafe { Self::new_unchecked("/") };

			/// Returns the byte lenght of the path.
			pub fn len(&self) -> usize {
				self.as_bytes().len()
			}

			/// Checks if the path is empty.
			///
			/// Returns `true` if the path has no segments.
			/// The absolute path `/` is empty.
			#[inline(always)]
			pub fn is_empty(&self) -> bool {
				self.as_bytes().is_empty() || self.as_bytes() == *b"/"
			}

			/// Checks if the path is absolute.
			///
			/// A path is absolute if it starts with a `/`.
			#[inline(always)]
			pub fn is_absolute(&self) -> bool {
				self.as_bytes().starts_with(b"/")
			}

			/// Checks if the path is relative.
			///
			/// A path is relative if it does not start with a `/`.
			#[inline(always)]
			pub fn is_relative(&self) -> bool {
				!self.is_absolute()
			}

			/// Returns the number of segments in the path.
			///
			/// This computes in linear time w.r.t the number of segments. It is
			/// equivalent to `path.segments().count()`.
			#[inline]
			pub fn segment_count(&self) -> usize {
				self.segments().count()
			}

			fn first_segment_offset(&self) -> usize {
				if self.is_absolute() { 1 } else { 0 }
			}

			pub fn first(&self) -> Option<&Segment> {
				if self.is_empty() {
					None
				} else {
					Some(unsafe { self.segment_at(self.first_segment_offset()).0 })
				}
			}

			pub fn last(&self) -> Option<&Segment> {
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
			unsafe fn segment_at(&self, offset: usize) -> (&Segment, usize) {
				let mut i = offset;

				let bytes = self.as_bytes();
				while i < bytes.len() && !matches!(bytes[i], b'/' | b'?' | b'#') {
					i += 1
				}

				(
					unsafe { Segment::new_unchecked_from_bytes(&bytes[offset..i]) },
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
			unsafe fn next_segment_from(&self, offset: usize) -> Option<(&Segment, usize)> {
				let bytes = self.as_bytes();
				if offset <= bytes.len() {
					Some(unsafe { self.segment_at(offset) })
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
			unsafe fn previous_segment_from(&self, offset: usize) -> Option<(&Segment, usize)> {
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
						Some((unsafe { self.segment_at(j) }.0, j))
					} else {
						Some((unsafe { self.segment_at(first_offset) }.0, first_offset))
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
			pub fn segments(&self) -> Segments<'_> {
				if self.is_empty() {
					Segments::Empty
				} else {
					Segments::NonEmpty {
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
			pub fn normalized_segments(&self) -> NormalizedSegments<'_> {
				NormalizedSegments::new(self)
			}

			#[inline]
			pub fn normalized(&self) -> PathBuf {
				let mut result: PathBuf = if self.is_absolute() {
					Self::EMPTY_ABSOLUTE.to_owned()
				} else {
					Self::EMPTY.to_owned()
				};

				let mut open = false;
				for segment in self.segments() {
					open = result.as_path_mut().push_inner(segment)
				}

				if open && !result.is_empty() {
					result.as_path_mut().lazy_push(Segment::EMPTY)
				}

				result
			}

			/// Returns the last segment of the path, if there is one, unless it is
			/// empty.
			///
			/// This does not consider the normalized version of the path, dot segments
			/// are preserved.
			#[inline]
			pub fn file_name(&self) -> Option<&Segment> {
				self.segments().next_back().filter(|s| !s.is_empty())
			}

			/// Returns the directory path, which is the path without the file name.
			///
			/// # Example
			///
			/// ```
			/// # use iref::uri::Path;
			/// assert_eq!(Path::new("/foo/bar").unwrap().directory(), "/foo/");
			/// assert_eq!(Path::new("/foo").unwrap().directory(), "/");
			/// assert_eq!(Path::new("//foo").unwrap().directory(), "//");
			/// assert_eq!(Path::new("/").unwrap().directory(), "/");
			/// ```
			pub fn directory(&self) -> &Self {
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
						unsafe { Self::new_unchecked_from_bytes(&bytes[..=i]) }
					}
				}
			}

			/// Returns the path without its final segment, if there is one.
			///
			/// ```
			/// # use iref::uri::Path;
			/// assert_eq!(Path::new("/foo/bar").unwrap().parent().unwrap(), "/foo");
			/// assert_eq!(Path::new("/foo").unwrap().parent().unwrap(), "/");
			/// assert_eq!(Path::new("//foo").unwrap().parent().unwrap(), "/./");
			/// assert_eq!(Path::new("/").unwrap().parent(), None);
			/// ```
			#[inline]
			pub fn parent(&self) -> Option<&Self> {
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
						unsafe { Some(Self::new_unchecked_from_bytes(b"/./")) }
					} else {
						unsafe { Some(Self::new_unchecked_from_bytes(&bytes[..end])) }
					}
				}
			}

			/// Returns the path without its final segment, if there is one.
			///
			/// ```
			/// # use iref::uri::Path;
			/// assert_eq!(Path::new("/foo/bar").unwrap().parent_or_empty(), "/foo");
			/// assert_eq!(Path::new("/foo").unwrap().parent_or_empty(), "/");
			/// assert_eq!(Path::new("//foo").unwrap().parent_or_empty(), "/./");
			/// assert_eq!(Path::new("/").unwrap().parent_or_empty(), "/");
			/// assert_eq!(Path::new("").unwrap().parent_or_empty(), "");
			/// ```
			#[inline]
			pub fn parent_or_empty(&self) -> &Self {
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
			/// use iref::iri::{Path, PathBuf};
			///
			/// let prefix = Path::new("/foo/bar").unwrap();
			/// let path = Path::new("/foo/bar/baz").unwrap();
			/// let suffix: PathBuf = path.suffix(prefix).unwrap();
			///
			/// assert_eq!(suffix.as_str(), "baz");
			/// ```
			#[inline]
			pub fn suffix(&self, prefix: &Self) -> Option<PathBuf> {
				if self.is_absolute() != prefix.is_absolute() {
					return None;
				}

				let mut buf: PathBuf = Default::default();
				let mut self_it = self.normalized_segments();
				let mut prefix_it = prefix.normalized_segments();

				loop {
					match (self_it.next(), prefix_it.next()) {
						(Some(self_seg), Some(prefix_seg))
							if self_seg.as_pct_str() == prefix_seg.as_pct_str() => {}
						(_, Some(_)) => return None,
						(Some(seg), None) => buf.as_path_mut().lazy_push(seg),
						(None, None) => break,
					}
				}

				Some(buf)
			}

			/// Checks if this path looks like a scheme.
			///
			/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
			/// valid scheme, of `false` otherwise.
			pub fn looks_like_scheme(&self) -> bool {
				crate::common::parse::looks_like_scheme(self.as_bytes())
			}
		}

		impl<'a> IntoIterator for &'a Path {
			type Item = &'a Segment;
			type IntoIter = Segments<'a>;

			#[inline]
			fn into_iter(self) -> Segments<'a> {
				self.segments()
			}
		}

		impl PartialEq for Path {
			#[inline]
			fn eq(&self, other: &Path) -> bool {
				if self.is_absolute() == other.is_absolute() {
					let self_segments = self.normalized_segments();
					let other_segments = other.normalized_segments();
					self_segments.len() == other_segments.len()
						&& self_segments.zip(other_segments).all(|(a, b)| a == b)
				} else {
					false
				}
			}
		}

		impl Eq for Path {}

		impl PartialOrd for Path {
			#[inline]
			fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl Ord for Path {
			#[inline]
			fn cmp(&self, other: &Self) -> std::cmp::Ordering {
				use std::cmp::Ordering;
				if self.is_absolute() == other.is_absolute() {
					let mut self_segments = self.normalized_segments();
					let mut other_segments = other.normalized_segments();

					loop {
						match (self_segments.next(), other_segments.next()) {
							(None, None) => return Ordering::Equal,
							(Some(_), None) => return Ordering::Greater,
							(None, Some(_)) => return Ordering::Less,
							(Some(a), Some(b)) => match a.cmp(b) {
								Ordering::Greater => return Ordering::Greater,
								Ordering::Less => return Ordering::Less,
								Ordering::Equal => (),
							},
						}
					}
				} else if self.is_absolute() {
					Ordering::Greater
				} else {
					Ordering::Less
				}
			}
		}

		impl std::hash::Hash for Path {
			#[inline]
			fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
				self.is_absolute().hash(hasher);
				self.normalized_segments().for_each(move |s| s.hash(hasher))
			}
		}

		pub enum Segments<'a> {
			Empty,
			NonEmpty {
				path: &'a Path,
				offset: usize,
				back_offset: usize,
			},
		}

		impl<'a> Iterator for Segments<'a> {
			type Item = &'a Segment;

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

		impl<'a> DoubleEndedIterator for Segments<'a> {
			fn next_back(&mut self) -> Option<Self::Item> {
				match self {
					Segments::Empty => None,
					Segments::NonEmpty {
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

		pub struct NormalizedSegments<'a>(smallvec::IntoIter<[&'a Segment; NORMALIZE_STACK_SIZE]>);

		impl<'a> NormalizedSegments<'a> {
			fn new(path: &'a Path) -> NormalizedSegments<'a> {
				let relative = path.is_relative();
				let mut stack = smallvec::SmallVec::<[&'a Segment; NORMALIZE_STACK_SIZE]>::new();

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

				NormalizedSegments(stack.into_iter())
			}
		}

		impl<'a> Iterator for NormalizedSegments<'a> {
			type Item = &'a Segment;

			fn size_hint(&self) -> (usize, Option<usize>) {
				self.0.size_hint()
			}

			#[inline]
			fn next(&mut self) -> Option<&'a Segment> {
				self.0.next()
			}
		}

		impl<'a> DoubleEndedIterator for NormalizedSegments<'a> {
			#[inline]
			fn next_back(&mut self) -> Option<Self::Item> {
				self.0.next_back()
			}
		}

		impl<'a> ExactSizeIterator for NormalizedSegments<'a> {}

		impl PathBuf {
			/// Returns a mutable reference to the interior bytes.
			///
			/// # Safety
			///
			/// This function is unsafe because the returned `&mut Vec` allows writing
			/// bytes which are not valid in a path. If this constraint is violated,
			/// using the original `PathBuf` after dropping the `&mut Vec` may violate
			/// memory safety, as the rest of the library assumes that `PathBuf` are
			/// valid paths.
			pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
				unsafe { self.0.as_mut_vec() }
			}

			pub fn as_path_mut(&mut self) -> PathMut<'_> {
				PathMut::from_path(self)
			}

			pub fn lazy_push(&mut self, segment: &Segment) {
				self.as_path_mut().lazy_push(segment)
			}

			/// Pop the last non-`..` segment of the path.
			///
			/// If the path is empty or ends in `..`, then a `..` segment
			/// will be added instead.
			pub fn pop(&mut self) -> bool {
				self.as_path_mut().pop()
			}

			pub fn clear(&mut self) {
				self.as_path_mut().clear()
			}

			/// Push the given segment to this path using the `.` and `..` segments semantics.
			#[inline]
			pub fn symbolic_push(&mut self, segment: &Segment) {
				self.as_path_mut().push(segment)
			}

			/// Append the given path to this path using the `.` and `..` segments semantics.
			///
			/// Note that this does not normalize the segments already in the path.
			/// For instance `'/a/b/.'.symbolc_append('../')` will return `/a/b/` and not
			/// `a/` because the semantics of `..` is applied on the last `.` in the path.
			#[inline]
			pub fn append<'s, P: IntoIterator<Item = &'s Segment>>(&mut self, path: P) {
				self.as_path_mut().append(path)
			}

			#[inline]
			pub fn normalize(&mut self) {
				self.as_path_mut().normalize()
			}
		}
	};
}

pub(crate) use path_impl;
