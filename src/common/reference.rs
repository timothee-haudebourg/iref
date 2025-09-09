macro_rules! reference {
	($name:literal: $uri:ident, $uri_buf:ident, $uri_ref:ident, $uri_ref_buf:ident) => {
		#[doc = $name]
		/// reference.
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::grammar::$uri_ref)]
		#[newtype(name = $name, name = " reference", ord([u8], &[u8], Vec<u8>, str, &str, String), owned($uri_ref_buf, derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)))]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct $uri_ref(str);

		#[doc = $name]
		/// reference parts.
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct Parts<'a> {
			pub scheme: Option<&'a Scheme>,
			pub authority: Option<&'a Authority>,
			pub path: &'a Path,
			pub query: Option<&'a Query>,
			pub fragment: Option<&'a Fragment>,
		}

		impl Default for &$uri_ref {
			fn default() -> Self {
				<$uri_ref>::EMPTY
			}
		}

		impl $uri_ref {
			pub const EMPTY: &'static Self = unsafe { Self::new_unchecked("") };

			pub fn parts(&self) -> Parts {
				let bytes = self.as_bytes();
				let ranges = crate::common::parse::reference_parts(bytes, 0);

				Parts {
					scheme: ranges
						.scheme
						.map(|r| unsafe { Scheme::new_unchecked_from_bytes(&bytes[r]) }),
					authority: ranges
						.authority
						.map(|r| unsafe { Authority::new_unchecked(&self.0[r]) }),
					path: unsafe { Path::new_unchecked(&self.0[ranges.path]) },
					query: ranges
						.query
						.map(|r| unsafe { Query::new_unchecked(&self.0[r]) }),
					fragment: ranges
						.fragment
						.map(|r| unsafe { Fragment::new_unchecked(&self.0[r]) }),
				}
			}

			/// Returns the scheme of the IRI reference, if any.
			#[inline]
			pub fn scheme(&self) -> Option<&Scheme> {
				let bytes = self.as_bytes();
				crate::common::parse::find_scheme(bytes, 0)
					.map(|range| unsafe { Scheme::new_unchecked_from_bytes(&bytes[range]) })
			}

			/// Returns the authority part of the IRI reference, if any.
			pub fn authority(&self) -> Option<&Authority> {
				let bytes = self.as_bytes();
				crate::common::parse::find_authority(bytes, 0)
					.ok()
					.map(|range| unsafe { Authority::new_unchecked_from_bytes(&bytes[range]) })
			}

			/// Returns the path of the IRI reference.
			pub fn path(&self) -> &Path {
				let bytes = self.as_bytes();
				let range = crate::common::parse::find_path(bytes, 0);
				unsafe { Path::new_unchecked_from_bytes(&bytes[range]) }
			}

			pub fn query(&self) -> Option<&Query> {
				let bytes = self.as_bytes();
				crate::common::parse::find_query(bytes, 0)
					.ok()
					.map(|range| unsafe { Query::new_unchecked_from_bytes(&bytes[range]) })
			}

			pub fn fragment(&self) -> Option<&Fragment> {
				let bytes = self.as_bytes();
				crate::common::parse::find_fragment(bytes, 0)
					.ok()
					.map(|range| unsafe { Fragment::new_unchecked_from_bytes(&bytes[range]) })
			}

			/// Get this IRI reference relatively to the given one.
			#[inline]
			pub fn relative_to(&self, other: &Self) -> $uri_ref_buf {
				let mut result = <$uri_ref_buf>::default();

				match (self.scheme(), other.scheme()) {
					(Some(a), Some(b)) if a == b => (),
					(Some(_), None) => (),
					(None, Some(_)) => (),
					(None, None) => (),
					_ => {
						return unsafe { <$uri_ref_buf>::new_unchecked(self.as_bytes().to_vec()) };
					}
				}

				match (self.authority(), other.authority()) {
					(Some(a), Some(b)) if a == b => (),
					(Some(_), None) => (),
					(None, Some(_)) => (),
					(None, None) => (),
					_ => {
						return unsafe { <$uri_ref_buf>::new_unchecked(self.as_bytes().to_vec()) };
					}
				}

				let mut self_segments = self.path().normalized_segments().peekable();
				let mut base_segments = other
					.path()
					.parent_or_empty()
					.normalized_segments()
					.peekable();

				if self.path().is_absolute() == other.path().is_absolute() {
					loop {
						match (self_segments.peek(), base_segments.peek()) {
							(Some(a), Some(b)) if a.as_pct_str() == b.as_pct_str() => {
								base_segments.next();
								self_segments.next();
							}
							_ => break,
						}
					}
				}

				for _segment in base_segments {
					result.path_mut().push(Segment::PARENT);
				}

				for segment in self_segments {
					result.path_mut().push(segment)
				}

				if (self.query().is_some() || self.fragment().is_some())
					&& Some(result.path().as_bytes()) == other.path().last().map(|s| s.as_bytes())
				{
					result.path_mut().clear()
				}

				result.set_query(self.query());
				result.set_fragment(self.fragment());

				result
			}

			/// Get the suffix of this URI, if any, with regard to the given prefix URI.
			///
			/// Returns `Some((suffix, query, fragment))` if this URI is of the form
			/// `prefix/suffix?query#fragment` where `prefix` is given as parameter.
			/// Returns `None` otherwise.
			/// If the `suffix` scheme or authority is different from this path, it will return `None`.
			///
			/// See [`Path::suffix`] for more details.
			#[inline]
			pub fn suffix(
				&self,
				prefix: impl AsRef<Self>,
			) -> Option<(PathBuf, Option<&Query>, Option<&Fragment>)> {
				let prefix = prefix.as_ref();
				if self.scheme() == prefix.scheme() && self.authority() == prefix.authority() {
					self.path()
						.suffix(prefix.path())
						.map(|suffix_path| (suffix_path, self.query(), self.fragment()))
				} else {
					None
				}
			}

			/// The IRI reference without the file name, query and fragment.
			///
			/// # Example
			/// ```
			/// # use iref::IriRef;
			/// let a = IriRef::new("https://crates.io/crates/iref?query#fragment").unwrap();
			/// let b = IriRef::new("https://crates.io/crates/iref/?query#fragment").unwrap();
			/// assert_eq!(a.base(), "https://crates.io/crates/");
			/// assert_eq!(b.base(), "https://crates.io/crates/iref/")
			/// ```
			#[inline]
			pub fn base(&self) -> &Self {
				let bytes = self.as_bytes();
				let path_range = crate::common::parse::find_path(bytes, 0);
				let path_start = path_range.start;
				let path = unsafe { Path::new_unchecked_from_bytes(&bytes[path_range]) };

				let directory_path = path.directory();
				let end = path_start + directory_path.len();
				unsafe { Self::new_unchecked_from_bytes(&bytes[..end]) }
			}

			/// Resolve the URI reference against the given *base URI*.
			///
			/// Return the resolved URI.
			/// See the [`UriRefBuf::resolve`] method for more information about the resolution process.
			#[inline]
			pub fn resolved(&self, base_iri: impl AsRef<$uri>) -> $uri_buf {
				let iri_ref = self.to_owned();
				iri_ref.into_resolved(base_iri)
			}
		}

		crate::common::owned_maybe_reference!($uri_ref_buf);

		impl $uri_ref_buf {
			/// Sets the scheme part.
			///
			/// If there is no authority and the start of the path looks like a scheme
			/// (e.g. `foo:`) then the path is prefixed with `./` to avoid being
			/// confused with a scheme.
			///
			/// # Example
			///
			/// ```
			/// use iref::{IriRefBuf, iri::Scheme};
			///
			/// let mut a = IriRefBuf::new("foo/bar".to_string()).unwrap();
			/// a.set_scheme(Some(Scheme::new(b"http").unwrap()));
			/// assert_eq!(a, "http:foo/bar");
			///
			/// let mut b = IriRefBuf::new("scheme://example.org/foo/bar".to_string()).unwrap();
			/// b.set_scheme(None);
			/// assert_eq!(b, "//example.org/foo/bar");
			///
			/// let mut c = IriRefBuf::new("scheme:foo:bar".to_string()).unwrap();
			/// c.set_scheme(None);
			/// assert_eq!(c, "./foo:bar");
			/// ```
			#[inline]
			pub fn set_scheme(&mut self, scheme: Option<&Scheme>) {
				match scheme {
					Some(new_scheme) => match crate::common::parse::find_scheme(self.as_bytes(), 0)
					{
						Some(scheme_range) => unsafe {
							self.replace(scheme_range, new_scheme.as_bytes());
						},
						None => unsafe {
							self.allocate(0..0, new_scheme.len() + 1);
							let bytes = self.as_mut_vec();
							bytes[0..new_scheme.len()].copy_from_slice(new_scheme.as_bytes());
							bytes[new_scheme.len()] = b':'
						},
					},
					None => {
						if let Some(scheme_range) =
							crate::common::parse::find_scheme(self.as_bytes(), 0)
						{
							let value: &[u8] =
								if self.authority().is_none() && self.path().looks_like_scheme() {
									// AMBIGUITY: The URI `http:foo:bar` would become
									//            `foo:bar`, but `foo` is not the scheme.
									// SOLUTION:  We change `foo:bar` to `./foo:bar`.
									b"./"
								} else {
									b""
								};

							unsafe {
								self.replace(scheme_range.start..(scheme_range.end + 1), value)
							}
						}
					}
				}
			}

			/// Resolve the URI/IRI reference.
			///
			/// ## Abnormal use of dot segments.
			///
			/// See <https://www.rfc-editor.org/errata/eid4547>
			pub fn resolve(&mut self, base_iri: impl AsRef<$uri>) {
				let base_iri = base_iri.as_ref();
				let parts = crate::common::parse::reference_parts(self.as_bytes(), 0);

				if parts.scheme.is_some() {
					self.path_mut().normalize();
				} else {
					self.set_scheme(Some(base_iri.scheme()));
					if parts.authority.is_some() {
						self.path_mut().normalize();
					} else if self.path().is_relative() && self.path().is_empty() {
						self.set_authority(base_iri.authority());
						self.set_path(base_iri.path());
						if self.query().is_none() {
							self.set_query(base_iri.query());
						}
					} else if self.path().is_absolute() {
						self.set_authority(base_iri.authority());
						self.path_mut().normalize();
					} else {
						self.set_authority(base_iri.authority());
						let mut path_buffer = <$uri_buf>::from_scheme(base_iri.scheme().to_owned()); // we set the scheme to avoid path disambiguation.
						path_buffer.set_authority(base_iri.authority()); // we set the authority to avoid path disambiguation.

						if base_iri.authority().is_some() && base_iri.path().is_empty() {
							path_buffer.set_path(Path::EMPTY_ABSOLUTE);
						} else {
							path_buffer.set_path(base_iri.path().parent_or_empty());
							path_buffer.path_mut().normalize();
						}

						path_buffer
							.path_mut()
							.symbolic_append(self.path().segments());

						self.set_path(path_buffer.path());
					}
				}
			}

			pub fn into_resolved(mut self, base_iri: impl AsRef<$uri>) -> $uri_buf {
				self.resolve(base_iri);
				unsafe { <$uri_buf>::new_unchecked(self.into_bytes()) }
			}
		}
	};
}

pub(crate) use reference;
