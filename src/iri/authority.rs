use std::ops::Range;
use std::{fmt, cmp};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use pct_str::PctStr;

use crate::parsing::{self, ParsedAuthority};
use super::Error;

pub struct Authority<'a> {
	/// Authority slice.
	pub(crate) data: &'a [u8],

	/// Authority positions.
	pub(crate) p: ParsedAuthority
}

impl<'a> Hash for Authority<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.userinfo().hash(hasher);
		self.host().hash(hasher);
		self.port().hash(hasher);
	}
}

impl<'a> Authority<'a> {
	pub fn as_ref(&self) -> &[u8] {
		self.data
	}

	/// Checks if the authority is empty.
	///
	/// It is empty if it has no user info, an empty host string, and no port.
	/// Note that empty user info or port is different from no user info and port.
	/// For instance, the authorities `@`, `:` and `@:` are not empty.
	pub fn is_empty(&self) -> bool {
		self.p.userinfo_len.is_none() && self.p.host_len == 0 && self.p.port_len.is_none()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.p.len()])
		}
	}

	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	pub fn userinfo(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.userinfo_len {
			unsafe {
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[0..len])))
			}
		} else {
			None
		}
	}

	pub fn host(&self) -> &PctStr {
		unsafe {
			let len = self.p.host_len;
			PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[0..len]))
		}
	}

	pub fn port(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.port_len {
			unsafe {
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[0..len])))
			}
		} else {
			None
		}
	}
}

impl<'a> TryFrom<&'a str> for Authority<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Authority<'a>, Error> {
		let parsed_authority = parsing::parse_authority(str.as_ref(), 0)?;
		if parsed_authority.len() < str.len() {
			Err(Error::InvalidAuthority)
		} else {
			Ok(Authority {
				data: str.as_ref(),
				p: parsed_authority
			})
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

impl<'a> cmp::PartialEq for Authority<'a> {
	fn eq(&self, other: &Authority) -> bool {
		self.userinfo() == other.userinfo() && self.port() == other.port() && self.host() == other.host()
	}
}

impl<'a> Eq for Authority<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Authority<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_pct_str() == *other
	}
}

pub struct AuthorityMut<'a> {
	/// The whole IRI data.
	pub(crate) data: &'a mut Vec<u8>,

	/// Authority positions.
	pub(crate) p: &'a mut ParsedAuthority
}

impl<'a> AuthorityMut<'a> {
	pub fn as_authority(&'a self) -> Authority<'a> {
		Authority {
			data: self.data.as_slice(),
			p: *self.p
		}
	}

	pub fn is_empty(&self) -> bool {
		self.as_authority().is_empty()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			let offset = self.p.offset;
			std::str::from_utf8_unchecked(&self.data[offset..(offset+self.p.len())])
		}
	}

	pub fn userinfo(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.userinfo_len {
			unsafe {
				let offset = self.p.offset;
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(self.data, self.p, false, range, content)
	}

	fn replace_before_authority(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(self.data, self.p, true, range, content)
	}

	pub fn set_raw_userinfo<S: AsRef<[u8]> + ?Sized>(&mut self, userinfo: Option<&S>) -> Result<(), Error> {
		let offset = self.p.offset;

		if userinfo.is_none() || userinfo.unwrap().as_ref().is_empty() {
			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset+userinfo_len+1), &[]);
			}

			self.p.userinfo_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		} else {
			let new_userinfo = userinfo.unwrap().as_ref();
			let new_userinfo_len = parsing::parse_userinfo(new_userinfo, 0)?;
			if new_userinfo_len != new_userinfo.len() {
				return Err(Error::Invalid);
			}

			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset+userinfo_len), new_userinfo);
			} else {
				self.replace(offset..offset, &[0x40]);
				self.replace(offset..offset, new_userinfo);
			}

			self.p.userinfo_len = Some(new_userinfo_len);
		}

		Ok(())
	}

	pub fn set_userinfo(&mut self, userinfo: Option<&str>) -> Result<(), Error> {
		self.set_raw_userinfo(userinfo)
	}

	pub fn host(&self) -> &PctStr {
		unsafe {
			let offset = self.p.host_offset();
			let len = self.p.host_len;
			PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
		}
	}

	pub fn set_host<S: AsRef<[u8]> + ?Sized>(&mut self, host: &S) -> Result<(), Error> {
		let offset = self.p.host_offset();
		let new_host = host.as_ref();
		let new_host_len = parsing::parse_host(new_host, 0)?;
		if new_host_len != new_host.len() {
			return Err(Error::Invalid);
		}

		self.replace(offset..(offset+self.p.host_len), new_host);
		self.p.host_len = new_host_len;

		if new_host_len == 0 {
			// Make the authority part implicit, if we can.
			self.make_implicit();
		}

		Ok(())
	}

	pub fn port(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.port_len {
			unsafe {
				let offset = self.p.port_offset();
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	pub fn set_raw_port<S: AsRef<[u8]> + ?Sized>(&mut self, port: Option<&S>) -> Result<(), Error> {
		let offset = self.p.port_offset();

		if port.is_none() || port.unwrap().as_ref().is_empty() {
			if let Some(port_len) = self.p.port_len {
				self.replace((offset-1)..(offset+port_len), &[]);
			}

			self.p.port_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		} else {
			let new_port = port.unwrap().as_ref();
			let new_port_len = parsing::parse_port(new_port, 0)?;
			if new_port_len != new_port.len() {
				return Err(Error::Invalid);
			}

			if let Some(port_len) = self.p.port_len {
				self.replace(offset..(offset+port_len), new_port);
			} else {
				self.replace(offset..offset, &[0x3a]);
				self.replace((offset+1)..(offset+1), new_port);
			}

			self.p.port_len = Some(new_port_len);
		}

		Ok(())
	}

	pub fn set_port(&mut self, port: Option<&str>) -> Result<(), Error> {
		self.set_raw_port(port)
	}

	/// Checks if there is an explicit authority delimiter `//` in the IRI.
	#[inline]
	pub fn is_explicit(&self) -> bool {
		if self.p.offset >= 2 {
			let end = self.p.offset;
			let start = end - 2;
			&self.data[start..end] == &[0x2f, 0x2f]
		} else {
			false
		}
	}

	pub fn is_implicit(&self) -> bool {
		!self.is_explicit()
	}

	/// Make sure there is an explicit authority delimiter `//` in the IRI.
	pub fn make_explicit(&mut self) {
		if !self.is_explicit() {
			let offset = self.p.offset;
			self.replace_before_authority(offset..offset, &[0x2f, 0x2f]);
		}
	}

	/// If possible, remove the authority delimiter `//`.
	///
	/// It has no effect if the authority is not empty,
	/// or if the path starts with `//`.
	/// Returns `true` if the authority is implicit after the method call,
	/// `false` otherwise.
	pub fn make_implicit(&mut self) -> bool {
		if self.is_explicit() {
			let is_path_authority_alike = {
				let path_offset = self.p.offset + self.p.len();
				self.data.len() >= path_offset + 2 && self.data[path_offset] == 0x2f && self.data[path_offset + 1] == 0x2f
			};

			if self.is_empty() && !is_path_authority_alike {
				let end = self.p.offset;
				let start = end - 2;
				self.replace_before_authority(start..end, &[]);
				true
			} else {
				false
			}
		} else {
			true
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{Iri, IriBuf};

	#[test]
	fn explicit_empty_with_authority_alike_path() {
		let iri = Iri::new("scheme:////").unwrap();
		let authority = iri.authority();

		assert!(authority.is_empty());
	}

	#[test]
	fn make_implicit() {
		let mut iri = IriBuf::new("scheme:///path").unwrap();
		let mut authority = iri.authority_mut();

		assert!(authority.make_implicit());
		assert_eq!(iri.as_str(), "scheme:/path");
	}

	#[test]
	fn make_implicit_edge_case() {
		let mut iri = IriBuf::new("scheme:////").unwrap();
		let mut authority = iri.authority_mut();

		assert!(!authority.make_implicit());
	}
}
