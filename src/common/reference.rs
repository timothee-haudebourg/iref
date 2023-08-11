use std::ops::Range;

use crate::uri::Scheme;
use super::{AuthorityImpl, PathImpl, QueryImpl, FragmentImpl, parse, AuthorityMutImpl, PathMutImpl, RiImpl, RiBufImpl};

pub struct RiRefParts<'a, T: ?Sized + RiRefImpl> {
	pub scheme: Option<&'a Scheme>,
	pub authority: Option<&'a T::Authority>,
	pub path: &'a T::Path,
	pub query: Option<&'a T::Query>,
	pub fragment: Option<&'a T::Fragment>,
}

impl<'a, T: ?Sized + RiRefBufImpl> RiRefParts<'a, T> {
	pub fn into_presence(self) -> RiRefPartsPresence {
		RiRefPartsPresence { scheme: self.scheme.is_some(), authority: self.authority.is_some(), query: self.query.is_some(), fragment: self.fragment.is_some() }
	}
}

pub struct RiRefPartsPresence {
	pub scheme: bool,
	pub authority: bool,
	pub query: bool,
	pub fragment: bool,
}

pub trait RiRefImpl {
	type Authority: ?Sized + AuthorityImpl;
	type Path: ?Sized + PathImpl;
	type Query: ?Sized + QueryImpl;
	type Fragment: ?Sized + FragmentImpl;

	fn as_bytes(&self) -> &[u8];

	fn parts(&self) -> RiRefParts<Self> {
		let bytes = self.as_bytes();
		let ranges = parse::reference_parts(bytes, 0);

		RiRefParts {
			scheme: ranges.scheme
				.map(|r| unsafe { Scheme::new_unchecked(&bytes[r]) }),
			authority: ranges.authority
				.map(|r| unsafe { Self::Authority::new_unchecked(&bytes[r]) }),
			path: unsafe { Self::Path::new_unchecked(&bytes[ranges.path]) },
			query: ranges.query
				.map(|r| unsafe { Self::Query::new_unchecked(&bytes[r]) }),
			fragment: ranges.fragment
				.map(|r| unsafe { Self::Fragment::new_unchecked(&bytes[r]) }),
		}
	}

	/// Returns the scheme of the IRI reference, if any.
	#[inline]
	fn scheme_opt(&self) -> Option<&Scheme> {
		let bytes = self.as_bytes();
		parse::find_scheme(bytes, 0).map(|range| unsafe {
			Scheme::new_unchecked(&bytes[range])
		})
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
		unsafe {
			Self::Path::new_unchecked(&bytes[range])
		}
	}

	fn query(&self) -> Option<&Self::Query> {
		let bytes = self.as_bytes();
		parse::find_query(bytes, 0).ok().map(|range| {
			unsafe { Self::Query::new_unchecked(&bytes[range]) }
		})
	}

	fn fragment(&self) -> Option<&Self::Fragment> {
		let bytes = self.as_bytes();
		parse::find_fragment(bytes, 0).ok().map(|range| {
			unsafe { Self::Fragment::new_unchecked(&bytes[range]) }
		})
	}
}

pub trait RiRefBufImpl: RiRefImpl {
	type Ri: ?Sized + RiImpl<
		Authority = Self::Authority,
		Path = Self::Path,
		Query = Self::Query,
		Fragment = Self::Fragment
	>;

	type RiBuf: ?Sized + RiBufImpl + RiImpl<
		Authority = Self::Authority,
		Path = Self::Path,
		Query = Self::Query,
		Fragment = Self::Fragment
	>;

	unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>;

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
			Some(new_scheme) => {
				match parse::find_scheme(self.as_bytes(), 0) {
					Some(scheme_range) => unsafe {
						self.replace(scheme_range, new_scheme.as_bytes());
					}
					None => unsafe {
						self.allocate(0..0, new_scheme.len()+1);
						let bytes = self.as_mut_vec();
						bytes[0..new_scheme.len()].copy_from_slice(new_scheme.as_bytes());
						bytes[new_scheme.len()] = b':'
					}
				}
			}
			None => {
				if let Some(scheme_range) = parse::find_scheme(self.as_bytes(), 0) {
					unsafe {
						self.replace(scheme_range.start..(scheme_range.end+1), b"")
					}
				}
			}
		}
	}

	#[inline]
	fn authority_mut(&mut self) -> Option<AuthorityMutImpl<Self::Authority>> {
		parse::find_authority(self.as_bytes(), 0).ok().map(|range| unsafe {
			AuthorityMutImpl::new(
				self.as_mut_vec(),
				range.start,
				range.end
			)
		})
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	#[inline]
	fn set_authority(&mut self, authority: Option<&Self::Authority>) {
		let bytes = self.as_bytes();
		match authority {
			Some(new_authority) => {
				match parse::find_authority(bytes, 0) {
					Ok(range) => unsafe {
						self.replace(range, new_authority.as_bytes())
					}
					Err(start) => unsafe {
						self.allocate(start..start, new_authority.len()+2);
						let bytes = self.as_mut_vec();
						let delim_end = start+2;
						bytes[start..delim_end].copy_from_slice(b"//");
						bytes[delim_end..(delim_end+new_authority.len())].copy_from_slice(new_authority.as_bytes())
					}
				}
			}
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
						self.replace((range.start-2)..range.end, value);
					}
				}
			}
		}
	}

	#[inline]
	fn path_mut(&mut self) -> PathMutImpl<Self::Path> {
		let range = parse::find_path(self.as_bytes(), 0);
		unsafe {
			PathMutImpl::new(
				self.as_mut_vec(),
				range.start,
				range.end
			)
		}
	}

	#[inline]
	fn set_path(&mut self, path: &Self::Path) {
		let range = parse::find_path(self.as_bytes(), 0);
		
		if path.as_bytes().starts_with(b"//") && self.authority().is_none() {
			// AMBIGUITY: The URI `http:old/path` would become
			//            `http://new_path`, but `//new_path` is not the
			//            authority.
			// SOLUTION:  We change `//new_path` to `/.//new_path`.
			unsafe {
				let start = range.start;
				let actual_start = start+2;
				self.allocate(range, path.len() + 2);
				let bytes = self.as_mut_vec();
				bytes[start..actual_start].copy_from_slice(b"/.");
				bytes[actual_start..(actual_start+path.len())].copy_from_slice(path.as_bytes())
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
			Some(new_query) => {
				match parse::find_query(self.as_bytes(), 0) {
					Ok(range) => unsafe {
						self.replace(range, new_query.as_bytes())
					}
					Err(start) => unsafe {
						self.allocate(start..start, new_query.len()+1);
						let bytes = self.as_mut_vec();
						let delim_end = start+1;
						bytes[start] = b'?';
						bytes[delim_end..(delim_end+new_query.len())].copy_from_slice(new_query.as_bytes())
					}
				}
			}
			None => {
				if let Ok(range) = parse::find_query(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start-1)..range.end, b"");
					}
				}
			}
		}
	}

	#[inline]
	fn set_fragment(&mut self, fragment: Option<&Self::Fragment>) {
		match fragment {
			Some(new_fragment) => {
				match parse::find_fragment(self.as_bytes(), 0) {
					Ok(range) => unsafe {
						self.replace(range, new_fragment.as_bytes())
					}
					Err(start) => unsafe {
						self.allocate(start..start, new_fragment.len()+1);
						let bytes = self.as_mut_vec();
						let delim_end = start+1;
						bytes[start] = b'#';
						bytes[delim_end..(delim_end+new_fragment.len())].copy_from_slice(new_fragment.as_bytes())
					}
				}
			}
			None => {
				if let Ok(range) = parse::find_fragment(self.as_bytes(), 0) {
					unsafe {
						self.replace((range.start-1)..range.end, b"");
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
		let has = self.parts().into_presence();

		if has.scheme {
			self.path_mut().normalize();
		} else {
			self.set_scheme(Some(base_iri.scheme()));
			if self.authority().is_some() {
				self.path_mut().normalize();
			} else {
				if self.path().is_relative() && self.path().is_empty() {
					self.set_path(base_iri.path());
					if self.query().is_none() {
						self.set_query(base_iri.query());
					}
				} else if self.path().is_absolute() {
					self.path_mut().normalize();
				} else {
					let mut path_buffer = Self::RiBuf::from_scheme(base_iri.scheme().to_owned()); // we set the scheme to avoid path disambiguation.
					path_buffer.set_authority(base_iri.authority()); // we set the authority to avoid path disambiguation.
					if base_iri.authority().is_some() && base_iri.path().is_empty() {
						path_buffer.set_path(Self::Path::EMPTY_ABSOLUTE);
					} else {
						path_buffer.set_path(base_iri.path().directory());
						path_buffer.path_mut().normalize();
					}
					path_buffer.path_mut().symbolic_append(self.path().segments());
					self.set_path(path_buffer.path());
				}
				self.set_authority(base_iri.authority());
			}
		}
	}
}