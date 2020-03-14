use std::ops::Range;
use std::convert::TryInto;
use std::fmt;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use pct_str::PctStr;
use crate::parsing::ParsedIriRef;
use crate::{Error, Iri, IriBuf, Scheme, Authority, AuthorityMut, Path, PathMut, Query, Fragment};
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

	pub fn as_iri(&self) -> Result<Iri, Error> {
		self.try_into()
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
		crate::replace(&mut self.data, &mut self.p.authority, false, range, content)
	}

	pub(crate) fn replace_before_authority(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(&mut self.data, &mut self.p.authority, true, range, content)
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
				self.replace_before_authority(0..scheme_len, new_scheme.as_ref());
			} else {
				self.replace_before_authority(0..0, &[0x3a]);
				self.replace_before_authority(0..0, new_scheme.as_ref());
			}

			self.p.scheme_len = Some(new_scheme.as_ref().len());
		} else {
			if let Some(scheme_len) = self.p.scheme_len {
				self.replace_before_authority(0..(scheme_len+1), &[]);
			}

			self.p.scheme_len = None;
		}
	}

	pub fn authority(&self) -> Authority {
		let offset = self.p.authority.offset;
		Authority {
			data: &self.data[offset..(offset+self.p.authority.len())],
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
		self.p.authority = new_parsed_authority;

		if authority.is_empty() {
			self.authority_mut().make_implicit();
		} else {
			self.authority_mut().make_explicit();
		}
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
	///
	/// ## Abnormal use of dot segments.
	///
	/// https://www.rfc-editor.org/errata/eid4547
	pub fn resolve<'b, Base: Into<Iri<'b>>>(&mut self, base_iri: Base) {
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
						path_buffer.path_mut().symbolic_append(self.path());
						if self.path().is_open() {
							path_buffer.path_mut().open();
						}
						self.set_path(path_buffer.path());
					}
				}
				self.set_authority(base_iri.authority());
			} else {
				self.path_mut().remove_dot_segments();
			}
		}
	}

	pub fn resolved<'b, Base: Into<Iri<'b>>>(&self, base_iri: Base) -> IriBuf {
		self.as_iri_ref().resolved(base_iri)
	}
}

impl fmt::Display for IriRefBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl fmt::Debug for IriRefBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri_ref().fmt(f)
	}
}

impl PartialEq for IriRefBuf {
	fn eq(&self, other: &IriRefBuf) -> bool {
		self.as_iri_ref() == other.as_iri_ref()
	}
}

impl Eq for IriRefBuf { }

impl<'a> PartialEq<IriRef<'a>> for IriRefBuf {
	fn eq(&self, iri_ref: &IriRef<'a>) -> bool {
		self.as_iri_ref() == *iri_ref
	}
}

impl<'a> PartialEq<Iri<'a>> for IriRefBuf {
	fn eq(&self, iri: &Iri<'a>) -> bool {
		self.as_iri_ref() == iri.as_iri_ref()
	}
}

impl PartialEq<IriBuf> for IriRefBuf {
	fn eq(&self, iri: &IriBuf) -> bool {
		self.as_iri_ref() == iri.as_iri_ref()
	}
}

impl PartialEq<str> for IriRefBuf {
	fn eq(&self, other: &str) -> bool {
		self.as_iri_ref() == other
	}
}

impl<'a> PartialEq<&'a str> for IriRefBuf {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_iri_ref() == *other
	}
}

impl PartialOrd for IriRefBuf {
	fn partial_cmp(&self, other: &IriRefBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl Ord for IriRefBuf {
	fn cmp(&self, other: &IriRefBuf) -> Ordering {
		self.as_iri_ref().cmp(&other.as_iri_ref())
	}
}

impl<'a> PartialOrd<IriRef<'a>> for IriRefBuf {
	fn partial_cmp(&self, other: &IriRef<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(other)
	}
}

impl<'a> PartialOrd<Iri<'a>> for IriRefBuf {
	fn partial_cmp(&self, other: &Iri<'a>) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl PartialOrd<IriBuf> for IriRefBuf {
	fn partial_cmp(&self, other: &IriBuf) -> Option<Ordering> {
		self.as_iri_ref().partial_cmp(&other.as_iri_ref())
	}
}

impl<'a> From<IriRef<'a>> for IriRefBuf {
	fn from(iri_ref: IriRef<'a>) -> IriRefBuf {
		let mut data = Vec::new();
		data.resize(iri_ref.as_ref().len(), 0);
		data.copy_from_slice(iri_ref.as_ref());

		IriRefBuf {
			p: iri_ref.p, data
		}
	}
}

impl<'a> From<&'a IriRef<'a>> for IriRefBuf {
	fn from(iri_ref: &'a IriRef<'a>) -> IriRefBuf {
		(*iri_ref).into()
	}
}

impl<'a> From<Iri<'a>> for IriRefBuf {
	fn from(iri: Iri<'a>) -> IriRefBuf {
		iri.as_iri_ref().into()
	}
}

impl<'a> From<&'a Iri<'a>> for IriRefBuf {
	fn from(iri: &'a Iri<'a>) -> IriRefBuf {
		(*iri).into()
	}
}

impl From<IriBuf> for IriRefBuf {
	fn from(iri: IriBuf) -> IriRefBuf {
		iri.0
	}
}

impl Hash for IriRefBuf {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_iri_ref().hash(hasher)
	}
}

#[cfg(test)]
mod tests {
	use crate::{Iri, IriRef};

	#[test]
	fn resolution_normal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("g:h", "g:h"),
			("g", "http://a/b/c/g"),
			("./g", "http://a/b/c/g"),
			("g/", "http://a/b/c/g/"),
			("/g", "http://a/g"),
			("//g", "http://g"),
			("?y", "http://a/b/c/d;p?y"),
			("g?y", "http://a/b/c/g?y"),
			("#s", "http://a/b/c/d;p?q#s"),
			("g#s", "http://a/b/c/g#s"),
			("g?y#s", "http://a/b/c/g?y#s"),
			(";x", "http://a/b/c/;x"),
			("g;x", "http://a/b/c/g;x"),
			("g;x?y#s", "http://a/b/c/g;x?y#s"),
			("", "http://a/b/c/d;p?q"),
			(".", "http://a/b/c/"),
			("./", "http://a/b/c/"),
			("..", "http://a/b/"),
			("../", "http://a/b/"),
			("../g", "http://a/b/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g")
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_abnormal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("../../../g", "http://a/g"),
			("../../../../g", "http://a/g"),
			("/./g", "http://a/g"),
			("/../g", "http://a/g"),
			("g.", "http://a/b/c/g."),
			(".g", "http://a/b/c/.g"),
			("g..", "http://a/b/c/g.."),
			("..g", "http://a/b/c/..g"),
			("./../g", "http://a/b/g"),
			("./g/.", "http://a/b/c/g/"),
			("g/./h", "http://a/b/c/g/h"),
			("g/../h", "http://a/b/c/h"),
			("g;x=1/./y", "http://a/b/c/g;x=1/y"),
			("g;x=1/../y", "http://a/b/c/y"),
			("g?y/./x", "http://a/b/c/g?y/./x"),
			("g?y/../x", "http://a/b/c/g?y/../x"),
			("g#s/./x", "http://a/b/c/g#s/./x"),
			("g#s/../x", "http://a/b/c/g#s/../x"),
			("http:g", "http:g")
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}
}
