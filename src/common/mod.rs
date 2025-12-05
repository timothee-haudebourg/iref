mod authority;
mod fragment;
pub(crate) mod parse;
mod path;
mod query;
mod reference;

pub(crate) use authority::*;
pub(crate) use fragment::*;
pub(crate) use path::*;
pub(crate) use query::*;
pub(crate) use reference::*;

macro_rules! borrowed {
	($(#[$meta:meta])* $name:literal: $m:ident, $uri:ident, $uri_buf:ident, $uri_ref:ident, $uri_ref_buf:ident) => {
		$(#[$meta])*
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(grammar::$uri)]
		#[newtype(name = $name, no_deref, ord([u8], &[u8], Vec<u8>, str, &str, String), owned($uri_buf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct $uri(str);

		impl $uri {
			pub fn parts(&self) -> Parts<'_> {
				let bytes = self.as_bytes();
				let ranges = crate::common::parse::parts(bytes, 0);

				Parts {
					scheme: unsafe { Scheme::new_unchecked_from_bytes(&bytes[ranges.scheme]) },
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

			/// Returns the scheme of the IRI.
			#[inline]
			pub fn scheme(&self) -> &Scheme {
				let bytes = self.as_bytes();
				let range = crate::common::parse::scheme(bytes, 0);
				unsafe { Scheme::new_unchecked_from_bytes(&bytes[range]) }
			}

			/// The
			#[doc = $name]
			/// without the file name, query and fragment.
			///
			/// # Example
			/// ```
			#[doc = concat!("# use iref::", stringify!($uri), ";")]
			#[doc = concat!("let a = ", stringify!($uri), "::new(\"https://crates.io/crates/iref?query#fragment\").unwrap();")]
			#[doc = concat!("let b = ", stringify!($uri), "::new(\"https://crates.io/crates/iref/?query#fragment\").unwrap();")]
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

			pub fn joined(&self, input: impl AsRef<$uri_ref>) -> $uri_buf {
                input.as_ref().resolved(self)
			}

			pub fn try_joined<'r>(&self, input: &'r str) -> Result<$uri_buf, <&'r $uri_ref as TryFrom<&'r str>>::Error> {
                $uri_ref::new(input).map(|r| self.joined(r))
			}
		}

		impl PartialEq for $uri {
			fn eq(&self, other: &Self) -> bool {
				self.parts() == other.parts()
			}
		}

		impl<'a> PartialEq<&'a $uri> for $uri {
			fn eq(&self, other: &&'a Self) -> bool {
				*self == **other
			}
		}

		impl Eq for $uri {}

		impl PartialOrd for $uri {
			fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl<'a> PartialOrd<&'a $uri> for $uri {
			fn partial_cmp(&self, other: &&'a Self) -> Option<std::cmp::Ordering> {
				self.partial_cmp(*other)
			}
		}

		impl Ord for $uri {
			fn cmp(&self, other: &Self) -> std::cmp::Ordering {
				self.parts().cmp(&other.parts())
			}
		}

		impl Hash for $uri {
			fn hash<H: hash::Hasher>(&self, state: &mut H) {
				self.parts().hash(state)
			}
		}

		#[doc = $name]
		/// parts.
		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct Parts<'a> {
			pub scheme: &'a Scheme,
			pub authority: Option<&'a Authority>,
			pub path: &'a Path,
			pub query: Option<&'a Query>,
			pub fragment: Option<&'a Fragment>,
		}

		crate::common::owned_maybe_reference!($uri_buf);

		impl $uri_buf {
			#[inline]
			pub fn from_scheme(scheme: SchemeBuf) -> Self {
				let mut bytes = scheme.into_bytes();
				bytes.push(b':');
				unsafe { Self::new_unchecked(bytes) }
			}

			/// Sets the scheme part.
			///
			/// # Example
			///
			/// ```
			#[doc = concat!("use iref::{", stringify!($uri_buf), ", ", stringify!($m), "::Scheme};")]
			///
			#[doc = concat!("let mut a = ", stringify!($uri_buf), "::new(\"http://example.org/path\".to_string()).unwrap();")]
			/// a.set_scheme(Scheme::new(b"https").unwrap());
			/// assert_eq!(a, "https://example.org/path");
			/// ```
			#[inline]
			pub fn set_scheme(&mut self, new_scheme: &Scheme) {
				let range = crate::common::parse::scheme(self.as_bytes(), 0);
				unsafe { self.replace(range, new_scheme.as_bytes()) }
			}

			/// Joins the given relative
			#[doc = $name]
			/// to this absolute
			#[doc = $name]
			/// .
			///
			/// This is quivalent to `input.resolve(self)`.
			pub fn join(&mut self, input: impl AsRef<$uri_ref>) {
			    let input = input.as_ref();
    			let parts = input.parts();

                match parts.scheme {
                    Some(scheme) => {
                        self.set_scheme(scheme);
        				self.set_authority(parts.authority);
                        self.set_and_normalize_path(parts.path);
                        self.set_query(parts.query);
                        self.set_fragment(parts.fragment);
                    }
                    None => {
                        match parts.authority {
                            Some(authority) => {
                                self.set_authority(Some(authority));
                                self.set_and_normalize_path(parts.path);
                                self.set_query(parts.query);
                                self.set_fragment(parts.fragment);
                            }
                            None => {
                                if parts.path.is_relative() && parts.path.is_empty() {
                                    if let Some(query) = parts.query {
                                        self.set_query(Some(query))
                                    }
                                } else if parts.path.is_absolute() {
                                    self.set_query(parts.query);
                                    self.set_and_normalize_path(parts.path);
                                } else {
                                    self.set_query(parts.query);
                                    self.path_mut().append(parts.path);
                                }

                                self.set_fragment(parts.fragment);
                            }
                        }
                    }
                }
			}

			pub fn try_join<'r>(&mut self, input: &'r str) -> Result<(), <&'r $uri_ref as TryFrom<&'r str>>::Error> {
				self.join($uri_ref::new(input)?);
				Ok(())
			}
		}
	};
}

pub(crate) use borrowed;

macro_rules! owned_maybe_reference {
	($this:ident) => {
		impl $this {
			#[inline]
			unsafe fn replace(&mut self, range: core::ops::Range<usize>, content: &[u8]) {
				crate::utils::replace(unsafe { self.as_mut_vec() }, range, content)
			}

			#[inline]
			unsafe fn allocate(&mut self, range: core::ops::Range<usize>, len: usize) {
				crate::utils::allocate_range(unsafe { self.as_mut_vec() }, range, len)
			}

			#[inline]
			pub fn authority_mut(&mut self) -> Option<AuthorityMut<'_>> {
				crate::common::parse::find_authority(self.as_bytes(), 0)
					.ok()
					.map(|range| unsafe {
						AuthorityMut::new(self.as_mut_vec(), range.start, range.end)
					})
			}

			/// Sets the authority part.
			///
			/// If the path is relative, this also turns it into an absolute path,
			/// since an authority cannot be followed by a relative path.
			///
			/// To avoid any ambiguity, if `authority` is `None` and the path starts
			/// with `//`, it will be changed into `/.//` as to not be interpreted as
			/// an authority part.
			///
			/// # Example
			///
			/// ```
			/// use iref::{IriRefBuf, iri::Authority};
			///
			/// let mut a = IriRefBuf::new("scheme:/path".to_string()).unwrap();
			/// a.set_authority(Some(Authority::new("example.org").unwrap()));
			/// assert_eq!(a, "scheme://example.org/path");
			///
			/// // When an authority is added before a relative path,
			/// // the path becomes absolute.
			/// let mut b = IriRefBuf::new("scheme:path".to_string()).unwrap();
			/// b.set_authority(Some(Authority::new("example.org").unwrap()));
			/// assert_eq!(b, "scheme://example.org/path");
			///
			/// // When an authority is removed and the path starts with `//`,
			/// // a `/.` prefix is added to the path to avoid any ambiguity.
			/// let mut c = IriRefBuf::new("scheme://example.org//path".to_string()).unwrap();
			/// c.set_authority(None);
			/// assert_eq!(c, "scheme:/.//path");
			/// ```
			#[inline]
			pub fn set_authority(&mut self, authority: Option<&Authority>) {
				let bytes = self.as_bytes();
				match authority {
					Some(new_authority) => match crate::common::parse::find_authority(bytes, 0) {
						Ok(range) => unsafe { self.replace(range, new_authority.as_bytes()) },
						Err(start) => {
							if !bytes[start..].starts_with(b"/") {
								// VALIDITY: When an authority is present, the path must
								//           be absolute.
								unsafe {
									self.allocate(start..start, new_authority.len() + 3);
									let bytes = self.as_mut_vec();
									let delim_end = start + 2;
									bytes[start..delim_end].copy_from_slice(b"//");
									bytes[delim_end..(delim_end + new_authority.len())]
										.copy_from_slice(new_authority.as_bytes());
									bytes[delim_end + new_authority.len()] = b'/';
								}
							} else {
								unsafe {
									self.allocate(start..start, new_authority.len() + 2);
									let bytes = self.as_mut_vec();
									let delim_end = start + 2;
									bytes[start..delim_end].copy_from_slice(b"//");
									bytes[delim_end..(delim_end + new_authority.len())]
										.copy_from_slice(new_authority.as_bytes())
								}
							}
						}
					},
					None => {
						if let Ok(range) = crate::common::parse::find_authority(bytes, 0) {
							let value: &[u8] = if bytes[range.end..].starts_with(b"//") {
								// AMBIGUITY: The URI `http://example.com//foo` would
								//            become `http://foo`, but `//foo` is not
								//            the authority.
								// SOLUTION:  We change `//foo` to `/.//foo`.
								b"/."
							} else {
								b""
							};

							unsafe {
								self.replace((range.start - 2)..range.end, value);
							}
						}
					}
				}
			}

			/// Tries to set the authority.
			///
			/// Same as [`Self::set_authority`] but accepts a `&str` instead of
			/// an [`&Authority`](Authority).
			pub fn try_set_authority<'s>(
				&mut self,
				authority: Option<&'s str>,
			) -> Result<(), InvalidAuthority<&'s str>> {
				self.set_authority(authority.map(TryInto::try_into).transpose()?);
				Ok(())
			}

			#[inline]
			pub fn path_mut(&mut self) -> PathMut<'_> {
				let range = crate::common::parse::find_path(self.as_bytes(), 0);
				unsafe { PathMut::new(self.as_mut_vec(), range.start, range.end) }
			}

			/// Sets the path part.
			///
			/// If there is an authority and the path is relative, this also turns it
			/// into an absolute path, since an authority cannot be followed by a
			/// relative path.
			///
			/// To avoid any ambiguity, if there is no authority and the path starts
			/// with `//`, it will be changed into `/.//` as to not be interpreted as
			/// an authority part. Similarly if there is no scheme nor authority and the
			/// beginning of the new path looks like a scheme, it is prefixed with `./`
			/// to not be confused with a scheme.
			///
			/// # Example
			///
			/// ```
			/// use iref::{IriRefBuf, iri::Path};
			///
			/// let mut a = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
			/// a.set_path(Path::new("/foo/bar").unwrap());
			/// assert_eq!(a, "http://example.org/foo/bar");
			///
			/// // If there is an authority and the new path is relative,
			/// // it is turned into an absolute path.
			/// let mut b = IriRefBuf::new("http://example.org/old/path".to_string()).unwrap();
			/// b.set_path(Path::new("relative/path").unwrap());
			/// assert_eq!(b, "http://example.org/relative/path");
			///
			/// // If there is no authority and the path starts with `//`,
			/// // it is prefixed with `/.` to avoid being confused with an authority.
			/// let mut c = IriRefBuf::new("http:old/path".to_string()).unwrap();
			/// c.set_path(Path::new("//foo/bar").unwrap());
			/// assert_eq!(c, "http:/.//foo/bar");
			///
			/// // If there is no authority nor scheme, and the path beginning looks
			/// // like a scheme, it is prefixed with `./` to avoid being confused with
			/// // a scheme.
			/// let mut d = IriRefBuf::new("old/path".to_string()).unwrap();
			/// d.set_path(Path::new("foo:bar").unwrap());
			/// assert_eq!(d, "./foo:bar");
			/// ```
			#[inline]
			pub fn set_path(&mut self, path: &Path) {
				self.path_mut().replace(path);
			}

			/// Tries to set the path.
			///
			/// Same as [`Self::set_path`] but accepts a `&str` instead of
			/// an [`&Path`](Path).
			pub fn try_set_path<'s>(&mut self, path: &'s str) -> Result<(), InvalidPath<&'s str>> {
				self.set_path(path.try_into()?);
				Ok(())
			}

			/// Sets and normalizes the path.
			pub fn set_and_normalize_path(&mut self, path: &Path) {
				self.set_path(path);
				self.path_mut().normalize();
			}

			#[inline]
			pub fn set_query(&mut self, query: Option<&Query>) {
				match query {
					Some(new_query) => match crate::common::parse::find_query(self.as_bytes(), 0) {
						Ok(range) => unsafe { self.replace(range, new_query.as_bytes()) },
						Err(start) => unsafe {
							self.allocate(start..start, new_query.len() + 1);
							let bytes = self.as_mut_vec();
							let delim_end = start + 1;
							bytes[start] = b'?';
							bytes[delim_end..(delim_end + new_query.len())]
								.copy_from_slice(new_query.as_bytes())
						},
					},
					None => {
						if let Ok(range) = crate::common::parse::find_query(self.as_bytes(), 0) {
							unsafe {
								self.replace((range.start - 1)..range.end, b"");
							}
						}
					}
				}
			}

			/// Tries to set the query part.
			///
			/// Same as [`Self::set_query`] but accepts a `&str` instead of
			/// an [`&Query`](Query).
			pub fn try_set_query<'s>(
				&mut self,
				query: Option<&'s str>,
			) -> Result<(), InvalidQuery<&'s str>> {
				self.set_query(query.map(TryInto::try_into).transpose()?);
				Ok(())
			}

			#[inline]
			pub fn set_fragment(&mut self, fragment: Option<&Fragment>) {
				match fragment {
					Some(new_fragment) => {
						match crate::common::parse::find_fragment(self.as_bytes(), 0) {
							Ok(range) => unsafe { self.replace(range, new_fragment.as_bytes()) },
							Err(start) => unsafe {
								self.allocate(start..start, new_fragment.len() + 1);
								let bytes = self.as_mut_vec();
								let delim_end = start + 1;
								bytes[start] = b'#';
								bytes[delim_end..(delim_end + new_fragment.len())]
									.copy_from_slice(new_fragment.as_bytes())
							},
						}
					}
					None => {
						if let Ok(range) = crate::common::parse::find_fragment(self.as_bytes(), 0) {
							unsafe {
								self.replace((range.start - 1)..range.end, b"");
							}
						}
					}
				}
			}

			/// Tries to set the fragment part.
			///
			/// Same as [`Self::set_fragment`] but accepts a `&str` instead of
			/// an [`&Fragment`](Fragment).
			pub fn try_set_fragment<'s>(
				&mut self,
				fragment: Option<&'s str>,
			) -> Result<(), InvalidFragment<&'s str>> {
				self.set_fragment(fragment.map(TryInto::try_into).transpose()?);
				Ok(())
			}
		}
	};
}

pub(crate) use owned_maybe_reference;
