pub use crate::{InvalidPort, Port, PortBuf};

mod host;
mod r#mut;
mod userinfo;

pub use host::*;
pub use r#mut::*;
pub use userinfo::*;

/// URI authority.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(crate::uri::grammar::Authority)]
#[newtype(ord([u8], &[u8], Vec<u8>, str, &str, String), owned(AuthorityBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Authority(str);

impl Authority {
	/// Returns references to the constituting parts of the authority.
	pub fn parts(&self) -> AuthorityParts<'_> {
		let bytes = self.as_bytes();

		let (user_info, host) = match crate::common::parse::user_info_or_host(bytes, 0) {
			(crate::common::parse::UserInfoOrHost::UserInfo, user_info_end) => {
				let host_start = user_info_end + 1;
				let host_end = crate::common::parse::host(bytes, host_start);
				(Some(0..user_info_end), host_start..host_end)
			}
			(crate::common::parse::UserInfoOrHost::Host, host_end) => (None, 0..host_end),
		};

		let (has_port, port_end) = crate::common::parse::port(bytes, host.end);
		let port = has_port.then_some((host.end + 1)..port_end);

		AuthorityParts {
			user_info: user_info.map(|r| unsafe { UserInfo::new_unchecked(&self.0[r]) }),
			host: unsafe { Host::new_unchecked(&self.0[host]) },
			port: port.map(|r| unsafe { Port::new_unchecked_from_bytes(&self.0.as_bytes()[r]) }),
		}
	}

	/// Returns a reference to the user information, if any.
	pub fn user_info(&self) -> Option<&UserInfo> {
		let bytes = self.as_bytes();
		crate::common::parse::find_user_info(bytes, 0)
			.map(|range| unsafe { UserInfo::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Returns a reference to the host name.
	pub fn host(&self) -> &Host {
		let bytes = self.as_bytes();
		let range = crate::common::parse::find_host(bytes, 0);
		unsafe { Host::new_unchecked_from_bytes(&bytes[range]) }
	}

	/// Returns a reference to the port, if any.
	pub fn port(&self) -> Option<&Port> {
		let bytes = self.as_bytes();
		crate::common::parse::find_port(bytes, 0)
			.map(|range| unsafe { Port::new_unchecked_from_bytes(&bytes[range]) })
	}
}

impl core::cmp::PartialEq for Authority {
	#[inline]
	fn eq(&self, other: &Authority) -> bool {
		self.parts() == other.parts()
	}
}

impl Eq for Authority {}

impl PartialOrd for Authority {
	#[inline]
	fn partial_cmp(&self, other: &Authority) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Authority {
	#[inline]
	fn cmp(&self, other: &Authority) -> core::cmp::Ordering {
		self.parts().cmp(&other.parts())
	}
}

impl core::hash::Hash for Authority {
	#[inline]
	fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
		self.parts().hash(hasher)
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityParts<'a> {
	pub user_info: Option<&'a UserInfo>,
	pub host: &'a Host,
	pub port: Option<&'a Port>,
}

impl AuthorityBuf {
	pub fn as_authority_mut(&mut self) -> AuthorityMut<'_> {
		let len = self.0.len();
		unsafe { AuthorityMut::new_unchecked(self.0.as_mut_vec(), 0..len) }
	}

	/// Replaces the value.
	#[inline]
	pub fn replace(&mut self, other: &Authority) -> &mut Self {
		self.as_authority_mut().replace(other);
		self
	}

	#[inline]
	pub fn try_replace<'s>(
		&mut self,
		other: &'s str,
	) -> Result<&mut Self, InvalidAuthority<&'s str>> {
		self.as_authority_mut().try_replace(other)?;
		Ok(self)
	}

	#[inline]
	pub fn set_user_info(&mut self, user_info: Option<&UserInfo>) -> &mut Self {
		self.as_authority_mut().set_user_info(user_info);
		self
	}

	#[inline]
	pub fn try_set_user_info<'s>(
		&mut self,
		user_info: Option<&'s str>,
	) -> Result<&mut Self, InvalidUserInfo<&'s str>> {
		self.as_authority_mut().try_set_user_info(user_info)?;
		Ok(self)
	}

	#[inline]
	pub fn set_host(&mut self, host: &Host) -> &mut Self {
		self.as_authority_mut().set_host(host);
		self
	}

	#[inline]
	pub fn try_set_host<'s>(&mut self, host: &'s str) -> Result<&mut Self, InvalidHost<&'s str>> {
		self.as_authority_mut().try_set_host(host)?;
		Ok(self)
	}

	#[inline]
	pub fn set_port(&mut self, port: Option<&Port>) -> &mut Self {
		self.as_authority_mut().set_port(port);
		self
	}

	#[inline]
	pub fn set_port_u32(&mut self, port: Option<u32>) -> &mut Self {
		self.as_authority_mut().set_port_u32(port);
		self
	}

	#[inline]
	pub fn try_set_port<'s>(
		&mut self,
		port: Option<&'s str>,
	) -> Result<&mut Self, InvalidPort<&'s str>> {
		self.as_authority_mut().try_set_port(port)?;
		Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use crate::Uri;

	#[test]
	fn explicit_empty_with_authority_alike_path() {
		let uri = Uri::new("scheme:////").unwrap();
		let authority = uri.authority();

		assert!(authority.unwrap().is_empty());
	}
}
