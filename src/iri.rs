use crate::parsing::{ParsedIri, ParsedAuthority};

use std::{fmt, cmp};
use log::*;

pub type Error = crate::parsing::Error;

pub struct Iri<'a> {
	data: &'a [u8],
	p: ParsedIri
}

pub struct Authority<'a> {
	data: &'a [u8],
	authority: &'a ParsedAuthority
}

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		Ok(Iri {
			data: buffer.as_ref(),
			p: ParsedIri::new(buffer)?
		})
	}

	pub fn scheme(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.p.scheme_len])
		}
	}

	pub fn authority(&'a self) -> Option<Authority<'a>> {
		if self.p.authority.is_empty() {
			None
		} else {
			Some(Authority {
				data: self.data,
				authority: &self.p.authority
			})
		}
	}

	pub fn path(&self) -> Option<&str> {
		if self.p.path_len > 0 {
			unsafe {
				let offset = self.p.authority.offset + self.p.authority.len();
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+self.p.path_len)]))
			}
		} else {
			None
		}
	}

	pub fn query(&self) -> Option<&str> {
		if let Some(len) = self.p.query_len {
			if len > 0 {
				unsafe {
					let offset = self.p.query_offset();
					Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
				}
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn fragment(&self) -> Option<&str> {
		if let Some(len) = self.p.fragment_len {
			if len > 0 {
				unsafe {
					let offset = self.p.fragment_offset();
					Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
				}
			} else {
				None
			}
		} else {
			None
		}
	}
}

impl<'a> Authority<'a> {
	pub fn as_str(&self) -> &str {
		unsafe {
			let offset = self.authority.offset;
			std::str::from_utf8_unchecked(&self.data[offset..(offset+self.authority.len())])
		}
	}

	pub fn userinfo(&self) -> Option<&str> {
		if let Some(len) = self.authority.userinfo_len {
			unsafe {
				let offset = self.authority.offset;
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
			}
		} else {
			None
		}
	}

	pub fn host(&self) -> Option<&str> {
		if let Some(len) = self.authority.host_len {
			unsafe {
				let offset = self.authority.host_offset();
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
			}
		} else {
			None
		}
	}

	pub fn port(&self) -> Option<&str> {
		if let Some(len) = self.authority.port_len {
			unsafe {
				let offset = self.authority.port_offset();
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
			}
		} else {
			None
		}
	}
}

impl<'a> fmt::Display for Authority<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Authority<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq<&'a str> for Authority<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}
