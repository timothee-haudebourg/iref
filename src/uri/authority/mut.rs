use std::ops::Range;

use super::{
	Authority, Host, InvalidAuthority, InvalidHost, InvalidPort, InvalidUserInfo, Port, PortBuf,
	UserInfo,
};

/// Mutable authority reference.
pub struct AuthorityMut<'a> {
	/// Arbitrary byte buffer containing the authority.
	data: &'a mut Vec<u8>,

	/// Authority range.
	range: Range<usize>,
}

impl<'a> AuthorityMut<'a> {
	/// Creates a new mutable authority part.
	#[inline]
	pub fn new(
		data: &'a mut Vec<u8>,
		range: Range<usize>,
	) -> Result<Self, InvalidAuthority<&'a [u8]>> {
		if Authority::validate_bytes(&data[range.clone()]) {
			Ok(unsafe { Self::new_unchecked(data, range) })
		} else {
			Err(InvalidAuthority(data))
		}
	}

	/// Creates a new mutable authority part.
	///
	/// # Safety
	///
	/// The buffer content between the range `start..end` must be a valid
	/// authority.
	#[inline]
	pub unsafe fn new_unchecked(data: &'a mut Vec<u8>, range: Range<usize>) -> Self {
		Self { data, range }
	}

	fn replace_bytes(&mut self, range: Range<usize>, content: &[u8]) {
		crate::utils::replace(self.data, range, content)
	}

	fn allocate_bytes(&mut self, range: Range<usize>, len: usize) {
		crate::utils::allocate_range(self.data, range, len)
	}

	#[inline]
	pub fn as_authority(&self) -> &Authority {
		unsafe { Authority::new_unchecked_from_bytes(&self.data[self.range.clone()]) }
	}

	#[inline]
	pub fn into_authority(self) -> &'a Authority {
		unsafe { Authority::new_unchecked_from_bytes(&self.data[self.range.clone()]) }
	}

	/// Replaces the value.
	#[inline]
	pub fn replace(&mut self, other: &Authority) {
		self.replace_bytes(self.range.clone(), other.as_bytes());
		self.range.end = self.range.start + other.len();
	}

	#[inline]
	pub fn try_replace<'s>(&mut self, other: &'s str) -> Result<(), InvalidAuthority<&'s str>> {
		self.replace(Authority::new(other)?);
		Ok(())
	}

	#[inline]
	pub fn set_user_info(&mut self, user_info: Option<&UserInfo>) {
		let bytes = &self.data[..self.range.end];

		match user_info {
			Some(new_userinfo) => {
				match crate::common::parse::find_user_info(bytes, self.range.start) {
					Some(userinfo_range) => {
						self.replace_bytes(userinfo_range, new_userinfo.as_bytes())
					}
					None => {
						let added_len = new_userinfo.len() + 1;
						self.allocate_bytes(self.range.start..self.range.start, added_len);
						self.data[self.range.start..(self.range.start + new_userinfo.len())]
							.copy_from_slice(new_userinfo.as_bytes());
						self.data[self.range.start + new_userinfo.len()] = b'@';
						self.range.end += added_len
					}
				}
			}
			None => {
				if let Some(userinfo_range) =
					crate::common::parse::find_user_info(bytes, self.range.start)
				{
					self.replace_bytes(userinfo_range.start..(userinfo_range.end + 1), b"");
					self.range.end -= userinfo_range.end - userinfo_range.start;
				}
			}
		}
	}

	#[inline]
	pub fn try_set_user_info<'s>(
		&mut self,
		user_info: Option<&'s str>,
	) -> Result<(), InvalidUserInfo<&'s str>> {
		self.set_user_info(user_info.map(TryInto::try_into).transpose()?);
		Ok(())
	}

	#[inline]
	pub fn set_host(&mut self, host: &Host) {
		let bytes = &self.data[..self.range.end];
		let range = crate::common::parse::find_host(bytes, self.range.start);
		let host_len = range.end - range.start;

		if host_len > host.len() {
			self.range.end -= host_len - host.len()
		} else {
			self.range.end -= host.len() - host_len
		}

		self.replace_bytes(range, host.as_bytes());
	}

	#[inline]
	pub fn try_set_host<'s>(&mut self, host: &'s str) -> Result<(), InvalidHost<&'s str>> {
		self.set_host(host.try_into()?);
		Ok(())
	}

	#[inline]
	pub fn set_port(&mut self, port: Option<&Port>) {
		let bytes = &self.data[..self.range.end];
		match port {
			Some(new_port) => match crate::common::parse::find_port(bytes, self.range.start) {
				Some(range) => self.replace_bytes(range, new_port.as_bytes()),
				None => {
					let added_len = new_port.len() + 1;
					self.allocate_bytes(self.range.end..self.range.end, added_len);
					self.data[self.range.end] = b':';
					self.data[(self.range.end + 1)..(self.range.end + added_len)]
						.copy_from_slice(new_port.as_bytes());
					self.range.end += added_len;
				}
			},
			None => {
				if let Some(port_range) = crate::common::parse::find_port(bytes, self.range.start) {
					self.replace_bytes((port_range.start - 1)..port_range.end, b"");
					self.range.end -= port_range.end - port_range.start;
				}
			}
		}
	}

	#[inline]
	pub fn set_port_u32(&mut self, port: Option<u32>) {
		let port: Option<PortBuf> = port.map(Into::into);
		self.set_port(port.as_deref());
	}

	#[inline]
	pub fn try_set_port<'s>(&mut self, port: Option<&'s str>) -> Result<(), InvalidPort<&'s str>> {
		self.set_port(port.map(TryInto::try_into).transpose()?);
		Ok(())
	}
}

impl<'a> ::core::ops::Deref for AuthorityMut<'a> {
	type Target = Authority;

	fn deref(&self) -> &Self::Target {
		self.as_authority()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn replace() {
		let mut data = b"http://user:pass@example.com:8080/path".to_vec();
		let mut authority_mut = AuthorityMut::new(&mut data, 7..33).unwrap();

		let new_authority = Authority::new("new_user:new_pass@new_host:9090").unwrap();
		authority_mut.replace(&new_authority);

		assert_eq!(
			authority_mut.as_authority(),
			"new_user:new_pass@new_host:9090"
		);
	}

	#[test]
	fn set_user_info() {
		let mut data = b"http://user:pass@example.com:8080/path".to_vec();
		let mut authority_mut = AuthorityMut::new(&mut data, 7..33).unwrap();

		let new_user_info = UserInfo::new("new_user:new_pass").unwrap();
		authority_mut.set_user_info(Some(&new_user_info));
		assert_eq!(
			authority_mut.as_authority().user_info().unwrap(),
			"new_user:new_pass"
		);

		authority_mut.set_user_info(None);
		assert!(authority_mut.as_authority().user_info().is_none());

		authority_mut.set_user_info(Some(&new_user_info));
		assert_eq!(
			authority_mut.as_authority().user_info().unwrap(),
			"new_user:new_pass"
		);
	}

	#[test]
	fn set_host() {
		let mut data = b"http://user:pass@example.com:8080/path".to_vec();
		let mut authority_mut = AuthorityMut::new(&mut data, 7..33).unwrap();

		let new_host = Host::new("new_host.org").unwrap();
		authority_mut.set_host(&new_host);
		assert_eq!(authority_mut.as_authority().host(), "new_host.org");
	}

	#[test]
	fn set_port() {
		let mut data = b"http://user:pass@example.com:8080/path".to_vec();
		let mut authority_mut = AuthorityMut::new(&mut data, 7..33).unwrap();

		let new_port = Port::new("9090").unwrap();
		authority_mut.set_port(Some(&new_port));
		assert_eq!(authority_mut.as_authority().port().unwrap(), "9090");

		authority_mut.set_port(None);
		assert!(authority_mut.as_authority().port().is_none());

		authority_mut.set_port(Some(&new_port));
		assert_eq!(authority_mut.as_authority().port().unwrap(), "9090");
	}
}
