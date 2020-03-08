use crate::parsing::{self, ParsedIri, ParsedAuthority};

use std::{fmt, cmp};
use std::ops::Range;
use log::*;

pub type Error = crate::parsing::Error;

pub struct Iri<'a> {
	data: &'a [u8],
	p: ParsedIri
}

pub struct IriBuf {
	data: Vec<u8>,
	p: ParsedIri
}

pub struct Authority<'a> {
	data: &'a [u8],
	authority: &'a ParsedAuthority
}

pub struct AuthorityMut<'a> {
	buffer: &'a mut IriBuf
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

	pub fn host(&self) -> &str {
		unsafe {
			let offset = self.authority.host_offset();
			let len = self.authority.host_len;
			std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])
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

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		Ok(Iri {
			data: buffer.as_ref(),
			p: ParsedIri::new(buffer)?
		})
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

	pub fn scheme(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.p.scheme_len])
		}
	}

	pub fn authority(&'a self) -> Authority<'a> {
		Authority {
			data: self.data,
			authority: &self.p.authority
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

impl IriBuf {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<IriBuf, Error> {
		Ok(IriBuf {
			data: Vec::from(buffer.as_ref()),
			p: ParsedIri::new(buffer)?
		})
	}

	pub fn as_iri(&self) -> Iri {
		Iri {
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

	pub fn scheme(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.p.scheme_len])
		}
	}

	pub fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		let range_len = range.end - range.start;

		// move the content around.
		if range_len != content.len() {
			let tail_len = self.data.len() - range.end; // the length of the content in the buffer after [range].
			let new_end = range.start + content.len();

			if range_len > content.len() { // shrink
				for i in 0..tail_len {
					self.data[new_end + i] = self.data[range.end + i];
				}

				self.data.resize(new_end + tail_len, 0);

				if self.p.authority.offset > range.end {
					let delta = range_len - content.len();
					self.p.authority.offset -= delta;
				}
			} else { // grow
				let tail_len = self.data.len() - range.end;

				self.data.resize(new_end + tail_len, 0);

				for i in 0..tail_len {
					self.data[new_end + tail_len - i - 1] = self.data[range.end + tail_len - i - 1];
				}

				if self.p.authority.offset > range.end {
					let delta = content.len() - range_len;
					self.p.authority.offset += delta;
				}
			}
		}

		// actually replace the content.
		for i in 0..content.len() {
			self.data[range.start + i] = content[i]
		}
	}

	/// Set the scheme of the IRI.
	///
	/// It must be a syntactically correct scheme. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_scheme<S: AsRef<[u8]> + ?Sized>(&mut self, scheme: &S) -> Result<(), Error> {
		let new_scheme = scheme.as_ref();
		let new_scheme_len = parsing::parse_scheme(new_scheme, 0)?;
		if new_scheme_len != new_scheme.len() {
			return Err(Error::Invalid);
		}
		self.replace(0..self.p.scheme_len, new_scheme);
		self.p.scheme_len = new_scheme_len;
		Ok(())
	}

	pub fn authority(&self) -> Authority {
		Authority {
			data: self.data.as_ref(),
			authority: &self.p.authority
		}
	}

	pub fn authority_mut(&mut self) -> AuthorityMut {
		AuthorityMut {
			buffer: self
		}
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_authority<S: AsRef<[u8]> + ?Sized>(&mut self, authority: &S) -> Result<(), Error> {
		let new_authority = authority.as_ref();
		let mut new_parsed_authority = parsing::parse_authority(new_authority, 0)?;
		if new_parsed_authority.len() != new_authority.len() {
			return Err(Error::Invalid);
		}
		let offset = self.p.authority.offset;
		new_parsed_authority.offset = offset;
		self.replace(offset..(offset+self.p.authority.len()), new_authority);
		self.p.authority = new_parsed_authority;
		Ok(())
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

	pub fn set_path<S: AsRef<[u8]> + ?Sized>(&mut self, path: &S) -> Result<(), Error> {
		let new_path = path.as_ref();
		let mut new_path_len = parsing::parse_path(new_path, 0)?;
		if new_path_len != new_path.len() {
			return Err(Error::Invalid);
		}
		let offset = self.p.path_offset();
		self.replace(offset..(offset+self.p.path_len), new_path);
		self.p.path_len = new_path_len;
		Ok(())
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

	pub fn set_raw_query<S: AsRef<[u8]> + ?Sized>(&mut self, query: Option<&S>) -> Result<(), Error> {
		let offset = self.p.query_offset();

		if query.is_none() || query.unwrap().as_ref().is_empty() {
			if let Some(query_len) = self.p.query_len {
				self.replace((offset-1)..(offset+query_len), &[]);
			}

			self.p.query_len = None;
		} else {
			let new_query = query.unwrap().as_ref();
			let mut new_query_len = parsing::parse_query(new_query, 0)?;
			if new_query_len != new_query.len() {
				return Err(Error::Invalid);
			}

			if let Some(query_len) = self.p.query_len {
				self.replace(offset..(offset+query_len), new_query);
			} else {
				self.replace(offset..offset, &[0x3f]);
				self.replace((offset+1)..(offset+1), new_query);
			}

			self.p.query_len = Some(new_query_len);
		}

		Ok(())
	}

	pub fn set_query(&mut self, query: Option<&str>) -> Result<(), Error> {
		self.set_raw_query(query)
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

	pub fn set_raw_fragment<S: AsRef<[u8]> + ?Sized>(&mut self, fragment: Option<&S>) -> Result<(), Error> {
		let offset = self.p.fragment_offset();

		if fragment.is_none() || fragment.unwrap().as_ref().is_empty() {
			if let Some(fragment_len) = self.p.fragment_len {
				self.replace((offset-1)..(offset+fragment_len), &[]);
			}

			self.p.fragment_len = None;
		} else {
			let new_fragment = fragment.unwrap().as_ref();
			let mut new_fragment_len = parsing::parse_fragment(new_fragment, 0)?;
			if new_fragment_len != new_fragment.len() {
				return Err(Error::Invalid);
			}

			if let Some(fragment_len) = self.p.fragment_len {
				self.replace(offset..(offset+fragment_len), new_fragment);
			} else {
				self.replace(offset..offset, &[0x23]);
				self.replace((offset+1)..(offset+1), new_fragment);
			}

			self.p.fragment_len = Some(new_fragment_len);
		}

		Ok(())
	}

	pub fn set_fragment(&mut self, fragment: Option<&str>) -> Result<(), Error> {
		self.set_raw_fragment(fragment)
	}
}

impl<'a> AuthorityMut<'a> {
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

	pub fn userinfo(&self) -> Option<&str> {
		if let Some(len) = self.authority().userinfo_len {
			unsafe {
				let offset = self.authority().offset;
				Some(std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)]))
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
		} else {
			let new_userinfo = userinfo.unwrap().as_ref();
			let mut new_userinfo_len = parsing::parse_userinfo(new_userinfo, 0)?;
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

	pub fn host(&self) -> &str {
		unsafe {
			let offset = self.buffer.p.authority.host_offset();
			let len = self.buffer.p.authority.host_len;
			std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)])
		}
	}

	pub fn set_host<S: AsRef<[u8]> + ?Sized>(&mut self, host: &S) -> Result<(), Error> {
		let offset = self.authority().host_offset();
		let new_host = host.as_ref();
		let mut new_host_len = parsing::parse_host(new_host, 0)?;
		if new_host_len != new_host.len() {
			return Err(Error::Invalid);
		}

		self.buffer.replace(offset..(offset+self.authority().host_len), new_host);
		self.authority_mut().host_len = new_host_len;

		Ok(())
	}

	pub fn port(&self) -> Option<&str> {
		if let Some(len) = self.buffer.p.authority.port_len {
			unsafe {
				let offset = self.buffer.p.authority.port_offset();
				Some(std::str::from_utf8_unchecked(&self.buffer.data[offset..(offset+len)]))
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
		} else {
			let new_port = port.unwrap().as_ref();
			let mut new_port_len = parsing::parse_port(new_port, 0)?;
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
}
