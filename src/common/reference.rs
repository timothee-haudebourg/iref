use std::ops::Range;

use super::{
	parse, AuthorityImpl, AuthorityMutImpl, FragmentImpl, PathImpl, PathMutImpl, QueryImpl,
	RiBufImpl, RiImpl, SegmentImpl,
};
use crate::uri::Scheme;

pub type Suffix<'a, R> = (
	<<R as RiRefImpl>::Path as PathImpl>::Owned,
	Option<&'a <R as RiRefImpl>::Query>,
	Option<&'a <R as RiRefImpl>::Fragment>,
);

pub trait RiRefImpl {
	type Authority: ?Sized + AuthorityImpl;
	type Path: ?Sized + PathImpl;
	type Query: ?Sized + QueryImpl;
	type Fragment: ?Sized + FragmentImpl;

	type RiRefBuf: Default
		+ RiRefBufImpl<
			Authority = Self::Authority,
			Path = Self::Path,
			Query = Self::Query,
			Fragment = Self::Fragment,
		>;

	fn as_bytes(&self) -> &[u8];

	/// Returns the scheme of the IRI reference, if any.
	#[inline]
	fn scheme_opt(&self) -> Option<&Scheme> {
		let bytes = self.as_bytes();
		parse::find_scheme(bytes, 0).map(|range| unsafe { Scheme::new_unchecked(&bytes[range]) })
	}

	/// Returns the authority part of the IRI reference, if any.
	fn authority(&self) -> Option<&Self::Authority> {
		let bytes = self.as_bytes();
		parse::find_authority(bytes, 0).ok().map(|range| unsafe {
			<Self::Authority as AuthorityImpl>::new_unchecked(&bytes[range])
		})
	}

	/// Returns the path of the IRI reference.
	fn path(&self) -> &Self::Path {
		let bytes = self.as_bytes();
		let range = parse::find_path(bytes, 0);
		unsafe { Self::Path::new_unchecked(&bytes[range]) }
	}

	fn query(&self) -> Option<&Self::Query> {
		let bytes = self.as_bytes();
		parse::find_query(bytes, 0)
			.ok()
			.map(|range| unsafe { Self::Query::new_unchecked(&bytes[range]) })
	}

	fn fragment(&self) -> Option<&Self::Fragment> {
		let bytes = self.as_bytes();
		parse::find_fragment(bytes, 0)
			.ok()
			.map(|range| unsafe { Self::Fragment::new_unchecked(&bytes[range]) })
	}

	/// Get this IRI reference relatively to the given one.
	#[inline]
	fn relative_to(&self, other: &Self) -> Self::RiRefBuf {
		let mut result = Self::RiRefBuf::default();

		match (self.scheme_opt(), other.scheme_opt()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => {
				return unsafe {
					<Self::RiRefBuf as RiRefBufImpl>::new_unchecked(self.as_bytes().to_vec())
				}
			}
		}

		match (self.authority(), other.authority()) {
			(Some(a), Some(b)) if a == b => (),
			(Some(_), None) => (),
			(None, Some(_)) => (),
			(None, None) => (),
			_ => {
				return unsafe {
					<Self::RiRefBuf as RiRefBufImpl>::new_unchecked(self.as_bytes().to_vec())
				}
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
			result
				.path_mut()
				.push(<<Self::Path as PathImpl>::Segment as SegmentImpl>::PARENT);
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

	#[inline]
	fn suffix(&self, prefix: &Self) -> Option<Suffix<Self>> {
		if self.scheme_opt() == prefix.scheme_opt() && self.authority() == prefix.authority() {
			self.path()
				.suffix(prefix.path())
				.map(|suffix_path| (suffix_path, self.query(), self.fragment()))
		} else {
			None
		}
	}

	#[inline]
	fn base(&self) -> &[u8] {
		let bytes = self.as_bytes();
		let path_range = parse::find_path(bytes, 0);
		let path_start = path_range.start;
		let path = unsafe { Self::Path::new_unchecked(&bytes[path_range]) };

		let directory_path = path.directory();
		let end = path_start + directory_path.len();
		&bytes[..end]
	}
}

pub trait RiRefBufImpl: Sized + RiRefImpl {
	type Ri: ?Sized
		+ RiImpl<
			Authority = Self::Authority,
			Path = Self::Path,
			Query = Self::Query,
			Fragment = Self::Fragment,
		>;

	type RiBuf: ?Sized
		+ RiBufImpl
		+ RiImpl<
			Authority = Self::Authority,
			Path = Self::Path,
			Query = Self::Query,
			Fragment = Self::Fragment,
		>;

	unsafe fn new_unchecked(bytes: Vec<u8>) -> Self;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>;

	fn into_bytes(self) -> Vec<u8>;

	#[inline]
	unsafe fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::utils::replace(self.as_mut_vec(), range, content)
	}

	#[inline]
	unsafe fn allocate(&mut self, range: Range<usize>, len: usize) {
		crate::utils::allocate_range(self.as_mut_vec(), range, len)
	}

	/// Set the scheme of the IRI reference.
	#[inline]
	fn set_scheme(&mut self, scheme: Option<&Scheme>) {
		match scheme {
			Some(new_scheme) => match parse::find_scheme(self.as_bytes(), 0) {
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
				if let Some(scheme_range) = parse::find_scheme(self.as_bytes(), 0) {
					let value: &[u8] =
						if self.authority().is_none() && self.path().looks_like_scheme() {
							// AMBIGUITY: The URI `http:foo:bar` would become
							//            `foo:bar`, but `foo` is not the scheme.
							// SOLUTION:  We change `foo:bar` to `./foo:bar`.
							b"./"
						} else {
							b""
						};

					unsafe { self.replace(scheme_range.start..(scheme_range.end + 1), value) }
				}
			}
		}
	}

	#[inline]
	fn authority_mut(&mut self) -> Option<AuthorityMutImpl<Self::Authority>> {
		parse::find_authority(self.as_bytes(), 0)
			.ok()
			.map(|range| unsafe {
				AuthorityMutImpl::new(self.as_mut_vec(), range.start, range.end)
			})
	}

	#[inline]
	fn set_authority(&mut self, authority: Option<&Self::Authority>) {
		let bytes = self.as_bytes();
		match authority {
			Some(new_authority) => match parse::find_authority(bytes, 0) {
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
				if let Ok(range) = parse::find_authority(bytes, 0) {
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

	#[inline]
	fn path_mut(&mut self) -> PathMutImpl<Self::Path> {
		let range = parse::find_path(self.as_bytes(), 0);
		unsafe { PathMutImpl::new(self.as_mut_vec(), range.start, range.end) }
	}

	#[inline]
	fn set_path(&mut self, path: &Self::Path) {
		let range = parse::find_path(self.as_bytes(), 0);

		let has_authority = self.authority().is_some();
		if !has_authority && path.as_bytes().starts_with(b"//") {
			// AMBIGUITY: The URI `http:old/path` would become
			//            `http://new_path`, but `//new_path` is not the
			//            authority.
			// SOLUTION:  We change `//new_path` to `/.//new_path`.
			unsafe {
				let start = range.start;
				let actual_start = start + 2;
				self.allocate(range, path.len() + 2);
				let bytes = self.as_mut_vec();
				bytes[start..actual_start].copy_from_slice(b"/.");
				bytes[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes())
			}
		} else if has_authority && path.is_relative() {
			// VALIDITY: When an authority is present, the path must be
			//           absolute.
			unsafe {
				let start = range.start;
				let actual_start = start + 1;
				self.allocate(range, path.len() + 1);
				let bytes = self.as_mut_vec();
				bytes[start] = b'/';
				bytes[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes())
			}
		} else if range.start == 0 && path.looks_like_scheme() {
			// AMBIGUITY: The URI `old/path` would become `new:path`, but `new`
			//            is not the scheme.
			// SOLUTION:  We change `new:path` to `./new:path`.
			unsafe {
				let start = range.start;
				let actual_start = start + 2;
				self.allocate(range, path.len() + 2);
				let bytes = self.as_mut_vec();
				bytes[start..actual_start].copy_from_slice(b"./");
				bytes[actual_start..(actual_start + path.len())].copy_from_slice(path.as_bytes())
			}
		} else {
			unsafe {
				self.replace(range, path.as_bytes());
			}
		}
	}

	#[inline]
	fn set_query(&mut self, query: Option<&Self::Query>) {
		match query {
			Some(new_query) => match parse::find_query(self.as_bytes(), 0) {
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
				if let Ok(range) = parse::find_query(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start - 1)..range.end, b"");
					}
				}
			}
		}
	}

	#[inline]
	fn set_fragment(&mut self, fragment: Option<&Self::Fragment>) {
		match fragment {
			Some(new_fragment) => match parse::find_fragment(self.as_bytes(), 0) {
				Ok(range) => unsafe { self.replace(range, new_fragment.as_bytes()) },
				Err(start) => unsafe {
					self.allocate(start..start, new_fragment.len() + 1);
					let bytes = self.as_mut_vec();
					let delim_end = start + 1;
					bytes[start] = b'#';
					bytes[delim_end..(delim_end + new_fragment.len())]
						.copy_from_slice(new_fragment.as_bytes())
				},
			},
			None => {
				if let Ok(range) = parse::find_fragment(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start - 1)..range.end, b"");
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
	fn resolve(&mut self, base_iri: &Self::Ri) {
		let parts = parse::reference_parts(self.as_bytes(), 0);

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
				let mut path_buffer = Self::RiBuf::from_scheme(base_iri.scheme().to_owned()); // we set the scheme to avoid path disambiguation.
				path_buffer.set_authority(base_iri.authority()); // we set the authority to avoid path disambiguation.

				if base_iri.authority().is_some() && base_iri.path().is_empty() {
					path_buffer.set_path(Self::Path::EMPTY_ABSOLUTE);
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

	fn into_resolved(mut self, base_iri: &Self::Ri) -> Self::RiBuf {
		self.resolve(base_iri);
		unsafe { Self::RiBuf::new_unchecked(self.into_bytes()) }
	}
}
