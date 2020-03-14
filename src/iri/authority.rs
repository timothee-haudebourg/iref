use std::ops::Range;
use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use pct_str::PctStr;

use crate::parsing::{self, ParsedAuthority};
use super::{Error, UserInfo, Host, Port};

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

	pub fn userinfo(&self) -> Option<UserInfo> {
		if let Some(len) = self.p.userinfo_len {
			Some(UserInfo {
				data: &self.data[0..len]
			})
		} else {
			None
		}
	}

	pub fn host(&self) -> Host {
		let len = self.p.host_len;
		let offset = self.p.host_offset() - self.p.offset;
		Host {
			data: &self.data[offset..(offset+len)]
		}
	}

	pub fn port(&self) -> Option<Port> {
		if let Some(len) = self.p.port_len {
			let offset = self.p.port_offset() - self.p.offset;
			Some(Port {
				data: &self.data[offset..(offset+len)]
			})
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

impl<'a> PartialOrd for Authority<'a> {
	fn partial_cmp(&self, other: &Authority<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Authority<'a> {
	fn cmp(&self, other: &Authority<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
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

	fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(self.data, self.p, false, range, content)
	}

	fn replace_before_authority(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(self.data, self.p, true, range, content)
	}

	pub fn userinfo(&self) -> Option<UserInfo> {
		if let Some(len) = self.p.userinfo_len {
			let offset = self.p.offset;
			Some(UserInfo {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	pub fn set_userinfo(&mut self, userinfo: Option<UserInfo>) {
		let offset = self.p.offset;

		if let Some(new_userinfo) = userinfo {
			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset+userinfo_len), new_userinfo.as_ref());
			} else {
				self.replace(offset..offset, &[0x40]);
				self.replace(offset..offset, new_userinfo.as_ref());
			}

			self.p.userinfo_len = Some(new_userinfo.as_ref().len());
		} else {
			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset+userinfo_len+1), &[]);
			}

			self.p.userinfo_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		}
	}

	pub fn host(&self) -> Host {
		let offset = self.p.host_offset();
		let len = self.p.host_len;
		Host {
			data: &self.data[offset..(offset+len)]
		}
	}

	pub fn set_host(&mut self, host: Host) {
		let offset = self.p.host_offset();
		self.replace(offset..(offset+self.p.host_len), host.as_ref());
		self.p.host_len = host.as_ref().len();

		if self.p.host_len == 0 {
			// Make the authority part implicit, if we can.
			self.make_implicit();
		} else {
			self.make_explicit();
		}
	}

	pub fn port(&self) -> Option<Port> {
		if let Some(len) = self.p.port_len {
			let offset = self.p.port_offset();
			Some(Port {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	pub fn set_port(&mut self, port: Option<Port>) {
		let offset = self.p.port_offset();

		if let Some(new_port) = port {
			if let Some(port_len) = self.p.port_len {
				self.replace(offset..(offset+port_len), new_port.as_ref());
			} else {
				self.replace(offset..offset, &[0x3a]);
				self.replace((offset+1)..(offset+1), new_port.as_ref());
			}

			self.p.port_len = Some(new_port.as_ref().len());
			// Make the authority part explicit.
			self.make_explicit();
		} else {
			if let Some(port_len) = self.p.port_len {
				self.replace((offset-1)..(offset+port_len), &[]);
			}

			self.p.port_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		}
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
