use std::ops::Range;
use pct_str::PctStr;
use std::convert::TryInto;
use crate::parsing::{self, ParsedIriRef};
use crate::{Error, Iri, Scheme, Authority, AuthorityMut, Path, PathMut, Query, Fragment};
use super::IriRef;

/// Owned IRI reference.
#[derive(Default, Clone)]
pub struct IriRefBuf {
	pub(crate) p: ParsedIriRef,
	pub(crate) data: Vec<u8>,
}

impl IriRefBuf {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<IriRefBuf, Error> {
		Ok(IriRefBuf {
			data: Vec::from(buffer.as_ref()),
			p: ParsedIriRef::new(buffer)?
		})
	}

	pub fn as_iri_ref(&self) -> IriRef {
		IriRef {
			data: self.data.as_ref(),
			p: self.p
		}
	}

	/// Length in bytes.
	pub fn len(&self) -> usize {
		self.p.len()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.len()])
		}
	}

	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

	pub(crate) fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(&mut self.data, &mut self.p.authority, range, content)
	}

	pub fn scheme(&self) -> Option<Scheme> {
		if let Some(scheme_len) = self.p.scheme_len {
			Some(Scheme {
				data: &self.data[0..scheme_len]
			})
		} else {
			None
		}
	}

	/// Set the scheme of the IRI.
	pub fn set_scheme(&mut self, scheme: Option<Scheme>) {
		if let Some(new_scheme) = scheme {
			if let Some(scheme_len) = self.p.scheme_len {
				self.replace(0..scheme_len, new_scheme.as_ref());
			} else {
				self.replace(0..0, &[0x3a]);
				self.replace(0..0, new_scheme.as_ref());
			}

			self.p.scheme_len = Some(new_scheme.as_ref().len());
		} else {
			if let Some(scheme_len) = self.p.scheme_len {
				self.replace(0..(scheme_len+1), &[]);
			}

			self.p.scheme_len = None;
		}
	}

	pub fn authority(&self) -> Authority {
		Authority {
			data: self.data.as_ref(),
			p: self.p.authority
		}
	}

	pub fn authority_mut(&mut self) -> AuthorityMut {
		AuthorityMut {
			data: &mut self.data,
			p: &mut self.p.authority
		}
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_authority(&mut self, authority: Authority) {
		let offset = self.p.authority.offset;
		let mut new_parsed_authority = authority.p;
		new_parsed_authority.offset = offset;
		self.replace(offset..(offset+self.p.authority.len()), authority.as_ref());
		self.p.authority = new_parsed_authority
	}

	pub fn path(&self) -> Path {
		let offset = self.p.authority.offset + self.p.authority.len();
		Path {
			data: &self.data[offset..(offset+self.p.path_len)]
		}
	}

	pub fn path_mut(&mut self) -> PathMut {
		PathMut {
			buffer: self
		}
	}

	pub fn set_path(&mut self, path: Path) {
		let offset = self.p.path_offset();
		self.replace(offset..(offset+self.p.path_len), path.as_ref());
		self.p.path_len = path.as_ref().len()
	}

	pub fn query(&self) -> Option<Query> {
		if let Some(len) = self.p.query_len {
			let offset = self.p.query_offset();
			Some(Query {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	pub fn set_query(&mut self, query: Option<Query>) {
		let offset = self.p.query_offset();

		if let Some(new_query) = query {
			if let Some(query_len) = self.p.query_len {
				self.replace(offset..(offset+query_len), new_query.as_ref());
			} else {
				self.replace(offset..offset, &[0x3f]);
				self.replace((offset+1)..(offset+1), new_query.as_ref());
			}

			self.p.query_len = Some(new_query.as_ref().len());
		} else {
			if let Some(query_len) = self.p.query_len {
				self.replace((offset-1)..(offset+query_len), &[]);
			}

			self.p.query_len = None;
		}
	}

	pub fn fragment(&self) -> Option<Fragment> {
		if let Some(len) = self.p.fragment_len {
			let offset = self.p.fragment_offset();
			Some(Fragment {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	pub fn set_fragment(&mut self, fragment: Option<Fragment>) {
		let offset = self.p.fragment_offset();

		if let Some(new_fragment) = fragment {
			if let Some(fragment_len) = self.p.fragment_len {
				self.replace(offset..(offset+fragment_len), new_fragment.as_ref());
			} else {
				self.replace(offset..offset, &[0x23]);
				self.replace((offset+1)..(offset+1), new_fragment.as_ref());
			}

			self.p.fragment_len = Some(new_fragment.as_ref().len());
		} else {
			if let Some(fragment_len) = self.p.fragment_len {
				self.replace((offset-1)..(offset+fragment_len), &[]);
			}

			self.p.fragment_len = None;
		}
	}

	/// Resolve the IRI reference.
	pub fn resolve<'b, Base: Into<Iri<'b>>>(&mut self, base_iri: Base) -> Result<(), Error> {
		let base_iri: Iri<'b> = base_iri.into();

		if self.scheme().is_some() {
			self.path_mut().remove_dot_segments();
		} else {
			self.set_scheme(Some(base_iri.scheme()));
			if self.authority().is_empty() {
				if self.path().is_relative() && self.path().is_empty() {
					self.set_path(base_iri.path());
					if self.query().is_none() {
						self.set_query(base_iri.query());
					}
				} else {
					if self.path().is_absolute() {
						self.path_mut().remove_dot_segments();
					} else {
						let mut path_buffer = IriRefBuf::default();
						if !base_iri.authority().is_empty() && base_iri.path().is_empty() {
							path_buffer.set_path("/".try_into().unwrap());
						} else {
							path_buffer.set_path(base_iri.path().directory());
						}
						path_buffer.path_mut().symbolic_append(self.path())?;
						self.set_path(path_buffer.path());
					}
				}
				self.set_authority(base_iri.authority());
			} else {
				self.path_mut().remove_dot_segments();
			}
		}

		Ok(())
	}
}
