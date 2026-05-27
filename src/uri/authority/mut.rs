use std::ops::Range;

use super::{
	Authority, Host, InvalidAuthority, InvalidHost, InvalidPort, InvalidUserInfo, Port, PortBuf,
	UserInfo,
};

/// Mutable reference to an authority component within a buffer.
///
/// This type allows in-place modification of an authority's components
/// (userinfo, host, port) without reallocating the entire URI.
pub struct AuthorityMut<'a> {
	/// Arbitrary byte buffer containing the authority.
	data: &'a mut Vec<u8>,

	/// Authority range.
	range: Range<usize>,
}

impl<'a> AuthorityMut<'a> {
	/// Creates a new mutable authority reference.
	///
	/// Returns an error if the bytes in the given range are not a valid authority.
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

	/// Creates a new mutable authority reference without validation.
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

	/// Returns an immutable reference to the authority.
	#[inline]
	pub fn as_authority(&self) -> &Authority {
		unsafe { Authority::new_unchecked_from_bytes(&self.data[self.range.clone()]) }
	}

	/// Consumes self and returns an immutable reference to the authority.
	#[inline]
	pub fn into_authority(self) -> &'a Authority {
		unsafe { Authority::new_unchecked_from_bytes(&self.data[self.range.clone()]) }
	}

	/// Replaces the entire authority value.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{Authority, AuthorityBuf};
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// authority.replace(Authority::new("other.com:8080").unwrap());
	///
	/// assert_eq!(authority, "other.com:8080");
	/// ```
	#[inline]
	pub fn replace(&mut self, other: &Authority) -> &mut Self {
		self.replace_bytes(self.range.clone(), other.as_bytes());
		self.range.end = self.range.start + other.len();
		self
	}

	/// Tries to replace the entire authority value from a string.
	///
	/// Returns an error if the string is not a valid authority.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::AuthorityBuf;
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// assert!(authority.try_replace("other.com:8080").is_ok());
	/// assert_eq!(authority, "other.com:8080");
	///
	/// assert!(authority.try_replace("not valid\0").is_err());
	/// ```
	#[inline]
	pub fn try_replace<'s>(
		&mut self,
		other: &'s str,
	) -> Result<&mut Self, InvalidAuthority<&'s str>> {
		self.replace(Authority::new(other)?);
		Ok(self)
	}

	/// Sets the user info component.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{AuthorityBuf, UserInfo};
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// authority.set_user_info(Some(UserInfo::new("user:pass").unwrap()));
	///
	/// assert_eq!(authority, "user:pass@example.org");
	/// ```
	#[inline]
	pub fn set_user_info(&mut self, user_info: Option<&UserInfo>) -> &mut Self {
		let bytes = &self.data[..self.range.end];

		match user_info {
			Some(new_userinfo) => {
				match crate::common::parse::find_user_info(bytes, self.range.start) {
					Some(userinfo_range) => {
						self.replace_bytes(userinfo_range, new_userinfo.as_bytes())
					}
					None => {
						let new_userinfo = new_userinfo.as_bytes();
						let added_len = new_userinfo.len() + 1;
						self.allocate_bytes(self.range.start..self.range.start, added_len);
						self.data[self.range.start..(self.range.start + new_userinfo.len())]
							.copy_from_slice(new_userinfo);
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

		self
	}

	/// Tries to set the user info from a string.
	///
	/// Returns an error if the string is not valid user info.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::AuthorityBuf;
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// assert!(authority.try_set_user_info(Some("user:pass")).is_ok());
	/// assert_eq!(authority, "user:pass@example.org");
	///
	/// assert!(authority.try_set_user_info(Some("invalid@user")).is_err());
	/// ```
	#[inline]
	pub fn try_set_user_info<'s>(
		&mut self,
		user_info: Option<&'s str>,
	) -> Result<&mut Self, InvalidUserInfo<&'s str>> {
		Ok(self.set_user_info(user_info.map(TryInto::try_into).transpose()?))
	}

	/// Sets the host component.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{AuthorityBuf, Host};
	///
	/// let mut authority = AuthorityBuf::new("example.org:8080".to_string()).unwrap();
	/// authority.set_host(Host::new("other.com").unwrap());
	///
	/// assert_eq!(authority, "other.com:8080");
	/// ```
	#[inline]
	pub fn set_host(&mut self, host: &Host) -> &mut Self {
		let bytes = &self.data[..self.range.end];
		let range = crate::common::parse::find_host(bytes, self.range.start);
		let host_len = range.end - range.start;
		let host = host.as_bytes();

		if host_len > host.len() {
			self.range.end -= host_len - host.len()
		} else {
			self.range.end -= host.len() - host_len
		}

		self.replace_bytes(range, host);
		self
	}

	/// Tries to set the host from a string.
	///
	/// Returns an error if the string is not a valid host.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::AuthorityBuf;
	///
	/// let mut authority = AuthorityBuf::new("example.org:8080".to_string()).unwrap();
	/// assert!(authority.try_set_host("other.com").is_ok());
	/// assert_eq!(authority, "other.com:8080");
	///
	/// assert!(authority.try_set_host("invalid/host").is_err());
	/// ```
	#[inline]
	pub fn try_set_host<'s>(&mut self, host: &'s str) -> Result<&mut Self, InvalidHost<&'s str>> {
		Ok(self.set_host(host.try_into()?))
	}

	/// Sets the port component.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::{AuthorityBuf, Port};
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// authority.set_port(Some(Port::new(b"443").unwrap()));
	///
	/// assert_eq!(authority, "example.org:443");
	/// ```
	#[inline]
	pub fn set_port(&mut self, port: Option<&Port>) -> &mut Self {
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

		self
	}

	/// Sets the port from a `u32` value.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::AuthorityBuf;
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// authority.set_port_u32(Some(8080));
	///
	/// assert_eq!(authority, "example.org:8080");
	/// ```
	#[inline]
	pub fn set_port_u32(&mut self, port: Option<u32>) -> &mut Self {
		let port: Option<PortBuf> = port.map(Into::into);
		self.set_port(port.as_deref())
	}

	/// Tries to set the port from a string.
	///
	/// Returns an error if the string is not a valid port.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::AuthorityBuf;
	///
	/// let mut authority = AuthorityBuf::new("example.org".to_string()).unwrap();
	/// assert!(authority.try_set_port(Some("8080")).is_ok());
	/// assert_eq!(authority, "example.org:8080");
	///
	/// assert!(authority.try_set_port(Some("not_a_port")).is_err());
	/// ```
	#[inline]
	pub fn try_set_port<'s>(
		&mut self,
		port: Option<&'s str>,
	) -> Result<&mut Self, InvalidPort<&'s str>> {
		Ok(self.set_port(port.map(TryInto::try_into).transpose()?))
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
		let vectors: &[(&[u8], Range<usize>, Option<&str>, &[u8])] = &[
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				Some("user"),
				b"http://user@example.com:8080/path",
			),
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				Some("user:pass"),
				b"http://user:pass@example.com:8080/path",
			),
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				None,
				b"http://example.com:8080/path",
			),
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				Some("%75ser:pass"),
				b"http://%75ser:pass@example.com:8080/path",
			),
		];

		for (input_data, range, user_info, expected) in vectors {
			let mut data = input_data.to_vec();
			let mut authority_mut = AuthorityMut::new(&mut data, range.clone()).unwrap();
			let new_user_info = user_info.map(|i| UserInfo::new(i).unwrap());
			authority_mut.set_user_info(new_user_info);
			assert_eq!(authority_mut.as_authority().user_info(), new_user_info);
			assert_eq!(data, expected.to_vec());
		}
	}

	#[test]
	fn set_host() {
		let vectors: &[(&[u8], Range<usize>, &str, &[u8])] = &[
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				"new_host.org",
				b"http://user:pass@new_host.org:8080/path",
			),
			(
				b"http://user:pass@example.com:8080/path",
				7..33,
				"%6Eew.org",
				b"http://user:pass@%6Eew.org:8080/path",
			),
			(
				b"http://user:pass@%65xample.com:8080/path",
				7..35,
				"example.com",
				b"http://user:pass@example.com:8080/path",
			),
		];

		for (input_data, range, host, expected) in vectors {
			let mut data = input_data.to_vec();
			let mut authority_mut = AuthorityMut::new(&mut data, range.clone()).unwrap();
			let new_host = Host::new(host).unwrap();
			authority_mut.set_host(&new_host);
			assert_eq!(authority_mut.as_authority().host(), new_host);
			assert_eq!(data, expected.to_vec());
		}
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
