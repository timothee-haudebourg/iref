use pct_str::PctStr;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::{cmp, fmt};

use super::{Error, Host, Port, UserInfo};
use crate::parsing::{self, ParsedAuthority};

pub struct Authority<'a> {
	/// Authority slice.
	pub(crate) data: &'a [u8],

	/// Authority positions.
	pub(crate) p: ParsedAuthority,
}

impl<'a> Hash for Authority<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.userinfo().hash(hasher);
		self.host().hash(hasher);
		self.port().hash(hasher);
	}
}

impl<'a> Authority<'a> {
	/// Checks if the authority is empty.
	///
	/// It is empty if it has no user info, an empty host string, and no port.
	/// Note that empty user info or port is different from no user info and port.
	/// For instance, the authorities `@`, `:` and `@:` are not empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.p.userinfo_len.is_none() && self.p.host_len == 0 && self.p.port_len.is_none()
	}

	/// Returns a reference to the byte representation of the authority.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		self.data
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.data[0..self.p.len()]) }
	}

	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	#[inline]
	pub fn userinfo(&self) -> Option<UserInfo> {
		self.p.userinfo_len.map(|len| UserInfo {
			data: &self.data[0..len],
		})
	}

	#[inline]
	pub fn host(&self) -> Host {
		let len = self.p.host_len;
		let offset = self.p.host_offset();
		Host {
			data: &self.data[offset..(offset + len)],
		}
	}

	#[inline]
	pub fn port(&self) -> Option<Port> {
		if let Some(len) = self.p.port_len {
			let offset = self.p.port_offset();
			Some(Port {
				data: &self.data[offset..(offset + len)],
			})
		} else {
			None
		}
	}
}

impl<'a> AsRef<[u8]> for Authority<'a> {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for Authority<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<Authority<'a>, Error> {
		let parsed_authority = parsing::parse_authority(str.as_ref(), 0)?;
		if parsed_authority.len() < str.len() {
			Err(Error::InvalidAuthority)
		} else {
			Ok(Authority {
				data: str.as_ref(),
				p: parsed_authority,
			})
		}
	}
}

impl<'a> fmt::Display for Authority<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Authority<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Authority<'a> {
	#[inline]
	fn eq(&self, other: &Authority) -> bool {
		self.userinfo() == other.userinfo()
			&& self.port() == other.port()
			&& self.host() == other.host()
	}
}

impl<'a> PartialOrd for Authority<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Authority<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Authority<'a> {
	#[inline]
	fn cmp(&self, other: &Authority<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl<'a> Eq for Authority<'a> {}

impl<'a> cmp::PartialEq<&'a str> for Authority<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_pct_str() == *other
	}
}

pub struct AuthorityMut<'a> {
	/// The whole IRI data.
	pub(crate) data: &'a mut Vec<u8>,

	pub(crate) offset: usize,

	/// Authority positions.
	pub(crate) p: &'a mut ParsedAuthority,
}

impl<'a> AuthorityMut<'a> {
	#[inline]
	pub fn as_authority(&'a self) -> Authority<'a> {
		Authority {
			data: self.data.as_slice(),
			p: *self.p,
		}
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.as_authority().is_empty()
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe {
			let offset = self.offset;
			std::str::from_utf8_unchecked(&self.data[offset..(offset + self.p.len())])
		}
	}

	#[inline]
	fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::replace(self.data, range, content)
	}

	#[inline]
	pub fn userinfo(&self) -> Option<UserInfo> {
		if let Some(len) = self.p.userinfo_len {
			let offset = self.offset;
			Some(UserInfo {
				data: &self.data[offset..(offset + len)],
			})
		} else {
			None
		}
	}

	#[inline]
	pub fn set_userinfo(&mut self, userinfo: Option<UserInfo>) {
		let offset = self.offset;

		if let Some(new_userinfo) = userinfo {
			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset + userinfo_len), new_userinfo.as_ref());
			} else {
				self.replace(offset..offset, &[0x40]);
				self.replace(offset..offset, new_userinfo.as_ref());
			}

			self.p.userinfo_len = Some(new_userinfo.as_ref().len());
		} else {
			if let Some(userinfo_len) = self.p.userinfo_len {
				self.replace(offset..(offset + userinfo_len + 1), &[]);
			}

			self.p.userinfo_len = None;
		}
	}

	#[inline]
	pub fn host(&self) -> Host {
		let offset = self.offset + self.p.host_offset();
		let len = self.p.host_len;
		Host {
			data: &self.data[offset..(offset + len)],
		}
	}

	#[inline]
	pub fn set_host(&mut self, host: Host) {
		let offset = self.offset + self.p.host_offset();
		self.replace(offset..(offset + self.p.host_len), host.as_ref());
		self.p.host_len = host.as_ref().len();
	}

	#[inline]
	pub fn port(&self) -> Option<Port> {
		if let Some(len) = self.p.port_len {
			let offset = self.offset + self.p.port_offset();
			Some(Port {
				data: &self.data[offset..(offset + len)],
			})
		} else {
			None
		}
	}

	#[inline]
	pub fn set_port(&mut self, port: Option<Port>) {
		let offset = self.offset + self.p.port_offset();

		if let Some(new_port) = port {
			if let Some(port_len) = self.p.port_len {
				self.replace(offset..(offset + port_len), new_port.as_ref());
			} else {
				self.replace(offset..offset, &[0x3a]);
				self.replace((offset + 1)..(offset + 1), new_port.as_ref());
			}

			self.p.port_len = Some(new_port.as_ref().len());
		} else {
			if let Some(port_len) = self.p.port_len {
				self.replace((offset - 1)..(offset + port_len), &[]);
			}

			self.p.port_len = None;
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::Iri;

	#[test]
	fn explicit_empty_with_authority_alike_path() {
		let iri = Iri::new("scheme:////").unwrap();
		let authority = iri.authority();

		assert!(authority.unwrap().is_empty());
	}
}
