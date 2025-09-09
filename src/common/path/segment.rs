macro_rules! segment {
	($name:literal) => {
		#[doc = $name]
		/// path segment.
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::super::grammar::Segment)]
		#[newtype(
			no_deref,
			ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
			owned(
				SegmentBuf,
				derive(PartialEq, Eq, PartialOrd, Ord, Hash)
			)
		)]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct Segment(str);

		impl Segment {
			pub const EMPTY: &'static Self = unsafe { Segment::new_unchecked("") };

			pub const CURRENT: &'static Self = unsafe { Segment::new_unchecked(".") };

			pub const PARENT: &'static Self = unsafe { Segment::new_unchecked("..") };

			/// Returns the segment as a percent-encoded string slice.
			#[inline]
			pub fn as_pct_str(&self) -> &pct_str::PctStr {
				unsafe { pct_str::PctStr::new_unchecked(self.as_bytes()) }
			}

			pub fn len(&self) -> usize {
				self.as_bytes().len()
			}

			pub fn is_empty(&self) -> bool {
				self.as_bytes().is_empty()
			}

			/// Checks if this segment looks like a scheme.
			///
			/// Returns `true` if it is of the form `prefix:suffix` where `prefix` is a
			/// valid scheme, of `false` otherwise.
			pub fn looks_like_scheme(&self) -> bool {
				crate::common::parse::looks_like_scheme(self.as_bytes())
			}
		}

		impl core::ops::Deref for Segment {
			type Target = pct_str::PctStr;

			fn deref(&self) -> &Self::Target {
				self.as_pct_str()
			}
		}

		impl PartialEq for Segment {
			fn eq(&self, other: &Self) -> bool {
				self.as_pct_str() == other.as_pct_str()
			}
		}

		impl Eq for Segment {}

		impl PartialOrd for Segment {
			fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl Ord for Segment {
			fn cmp(&self, other: &Self) -> core::cmp::Ordering {
				self.as_pct_str().cmp(other.as_pct_str())
			}
		}

		impl core::hash::Hash for Segment {
			fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
				self.as_pct_str().hash(state)
			}
		}
	};
}

pub(crate) use segment;
