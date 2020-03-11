pub struct Authority<'a> {
	data: &'a [u8],
	authority: &'a ParsedAuthority
}

impl<'a> Hash for Authority<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.userinfo().hash(hasher);
		self.host().hash(hasher);
		self.port().hash(hasher);
	}
}

impl<'a> Authority<'a> {
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
	buffer: &'a mut IriBuf
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
		let mut new_host_len = parsing::parse_host(new_host, 0)?;
		if new_host_len != new_host.len() {
			return Err(Error::Invalid);
		}

		self.buffer.replace(offset..(offset+self.authority().host_len), new_host);
		self.authority_mut().host_len = new_host_len;

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
