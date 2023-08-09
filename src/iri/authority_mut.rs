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
				self.replace(offset..offset, b"@");
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
		self.p.host_len = host.as_bytes().len();
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
				self.replace(offset..offset, b":");
				self.replace((offset + 1)..(offset + 1), new_port.as_ref());
			}

			self.p.port_len = Some(new_port.as_bytes().len());
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
