use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use pct_str::PctStr;

use crate::parsing::{self, ParsedAuthority};
use super::{Error, IriBuf};

pub struct Authority<'a> {
	/// IRI data.
	///
	/// Note that this not only includes the authority slice,
	/// but the whole IRI slice.
	pub(crate) data: &'a [u8],

	/// Authority positions.
	pub(crate) authority: &'a ParsedAuthority
}

impl<'a> Hash for Authority<'a> {
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
	pub fn is_empty(&self) -> bool {
		self.authority.userinfo_len.is_none() && self.authority.host_len == 0 && self.authority.port_len.is_none()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			let offset = self.authority.offset;
			std::str::from_utf8_unchecked(&self.data[offset..(offset+self.authority.len())])
		}
	}

	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	pub fn userinfo(&self) -> Option<&PctStr> {
		if let Some(len) = self.authority.userinfo_len {
			unsafe {
				let offset = self.authority.offset;
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	pub fn host(&self) -> &PctStr {
		unsafe {
			let offset = self.authority.host_offset();
			let len = self.authority.host_len;
			PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
		}
	}

	pub fn port(&self) -> Option<&PctStr> {
		if let Some(len) = self.authority.port_len {
			unsafe {
				let offset = self.authority.port_offset();
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	/// Checks if there is an explicit authority delimiter `//` in the IRI.
	#[inline]
	pub fn is_explicit(&self) -> bool {
		if self.authority.offset >= 2 {
			let end = self.authority.offset;
			let start = end - 2;
			&self.data[start..end] == &[0x2f, 0x2f]
		} else {
			false
		}
	}

	pub fn is_implicit(&self) -> bool {
		!self.is_explicit()
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
	pub(crate) buffer: &'a mut IriBuf
}

impl<'a> AuthorityMut<'a> {
	pub fn is_empty(&self) -> bool {
		self.buffer.authority().is_empty()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			let offset = self.buffer.p.authority.offset;
			std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+self.buffer.p.authority.len())])
		}
	}

	fn authority(&self) -> &ParsedAuthority {
		&self.buffer.p.authority
	}

	fn authority_mut(&mut self) -> &mut ParsedAuthority {
		&mut self.buffer.p.authority
	}

	pub fn userinfo(&self) -> Option<&PctStr> {
		if let Some(len) = self.authority().userinfo_len {
			unsafe {
				let offset = self.authority().offset;
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	pub fn set_raw_userinfo<S: AsRef<[u8]> + ?Sized>(&mut self, userinfo: Option<&S>) -> Result<(), Error> {
		let offset = self.authority().offset;

		if userinfo.is_none() || userinfo.unwrap().as_ref().is_empty() {
			if let Some(userinfo_len) = self.authority().userinfo_len {
				self.buffer.replace(offset..(offset+userinfo_len+1), &[]);
			}

			self.authority_mut().userinfo_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		} else {
			let new_userinfo = userinfo.unwrap().as_ref();
			let new_userinfo_len = parsing::parse_userinfo(new_userinfo, 0)?;
			if new_userinfo_len != new_userinfo.len() {
				return Err(Error::Invalid);
			}

			if let Some(userinfo_len) = self.authority().userinfo_len {
				self.buffer.replace(offset..(offset+userinfo_len), new_userinfo);
			} else {
				self.buffer.replace(offset..offset, &[0x40]);
				self.buffer.replace(offset..offset, new_userinfo);
			}

			self.authority_mut().userinfo_len = Some(new_userinfo_len);
		}

		Ok(())
	}

	pub fn set_userinfo(&mut self, userinfo: Option<&str>) -> Result<(), Error> {
		self.set_raw_userinfo(userinfo)
	}

	pub fn host(&self) -> &PctStr {
		unsafe {
			let offset = self.buffer.p.authority.host_offset();
			let len = self.buffer.p.authority.host_len;
			PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)]))
		}
	}

	pub fn set_host<S: AsRef<[u8]> + ?Sized>(&mut self, host: &S) -> Result<(), Error> {
		let offset = self.authority().host_offset();
		let new_host = host.as_ref();
		let new_host_len = parsing::parse_host(new_host, 0)?;
		if new_host_len != new_host.len() {
			return Err(Error::Invalid);
		}

		self.buffer.replace(offset..(offset+self.authority().host_len), new_host);
		self.authority_mut().host_len = new_host_len;

		if new_host_len == 0 {
			// Make the authority part implicit, if we can.
			self.make_implicit();
		}

		Ok(())
	}

	pub fn port(&self) -> Option<&PctStr> {
		if let Some(len) = self.buffer.p.authority.port_len {
			unsafe {
				let offset = self.buffer.p.authority.port_offset();
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)])))
			}
		} else {
			None
		}
	}

	pub fn set_raw_port<S: AsRef<[u8]> + ?Sized>(&mut self, port: Option<&S>) -> Result<(), Error> {
		let offset = self.authority().port_offset();

		if port.is_none() || port.unwrap().as_ref().is_empty() {
			if let Some(port_len) = self.authority().port_len {
				self.buffer.replace((offset-1)..(offset+port_len), &[]);
			}

			self.authority_mut().port_len = None;
			// Make the authority part implicit, if we can.
			self.make_implicit();
		} else {
			let new_port = port.unwrap().as_ref();
			let new_port_len = parsing::parse_port(new_port, 0)?;
			if new_port_len != new_port.len() {
				return Err(Error::Invalid);
			}

			if let Some(port_len) = self.authority().port_len {
				self.buffer.replace(offset..(offset+port_len), new_port);
			} else {
				self.buffer.replace(offset..offset, &[0x3a]);
				self.buffer.replace((offset+1)..(offset+1), new_port);
			}

			self.authority_mut().port_len = Some(new_port_len);
		}

		Ok(())
	}

	pub fn set_port(&mut self, port: Option<&str>) -> Result<(), Error> {
		self.set_raw_port(port)
	}

	/// Checks if there is an explicit authority delimiter `//` in the IRI.
	pub fn is_explicit(&self) -> bool {
		self.buffer.authority().is_explicit()
	}

	pub fn is_implicit(&self) -> bool {
		self.buffer.authority().is_implicit()
	}

	/// Make sure there is an explicit authority delimiter `//` in the IRI.
	pub fn make_explicit(&mut self) {
		if !self.is_explicit() {
			let offset = self.buffer.p.scheme_len + 1;
			self.buffer.replace(offset..offset, &[0x2f, 0x2f]);
			self.buffer.p.authority.offset += 2;
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
			if self.is_empty() && !self.buffer.path().is_authority_alike() {
				let start = self.buffer.p.scheme_len + 1;
				let end = start + 2;
				self.buffer.replace(start..end, &[]);
				self.buffer.p.authority.offset -= 2;
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
		assert!(authority.is_explicit());
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
