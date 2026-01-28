#[cfg(feature = "std")]
pub use crate::PortBuf;
pub use crate::{InvalidPort, Port};

mod host;
#[cfg(feature = "std")]
mod r#mut;
mod userinfo;

pub use host::*;
#[cfg(feature = "std")]
pub use r#mut::*;
pub use userinfo::*;

/// URI authority component.
///
/// The authority component of a URI contains the host, and optionally
/// a userinfo prefix and a port suffix: `[userinfo@]host[:port]`.
///
/// # Example
///
/// ```rust
/// use iref::uri::Authority;
///
/// let authority = Authority::new("user:pass@example.org:8080").unwrap();
///
/// assert_eq!(authority.user_info().unwrap(), "user:pass");
/// assert_eq!(authority.host(), "example.org");
/// assert_eq!(authority.port().unwrap(), "8080");
/// ```
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(crate::uri::grammar::Authority)]
#[newtype(ord([u8], &[u8], str, &str))]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String), owned(AuthorityBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Authority(str);

impl Authority {
	/// Returns all the parts of this authority.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Authority;
	///
	/// let authority = Authority::new("user@example.org:443").unwrap();
	/// let parts = authority.parts();
	///
	/// assert_eq!(parts.user_info.unwrap(), "user");
	/// assert_eq!(parts.host, "example.org");
	/// assert_eq!(parts.port.unwrap(), "443");
	/// ```
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
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Authority;
	///
	/// let with_userinfo = Authority::new("user:pass@example.org").unwrap();
	/// assert_eq!(with_userinfo.user_info().unwrap(), "user:pass");
	///
	/// let without = Authority::new("example.org").unwrap();
	/// assert!(without.user_info().is_none());
	/// ```
	pub fn user_info(&self) -> Option<&UserInfo> {
		let bytes = self.as_bytes();
		crate::common::parse::find_user_info(bytes, 0)
			.map(|range| unsafe { UserInfo::new_unchecked_from_bytes(&bytes[range]) })
	}

	/// Returns a reference to the host.
	///
	/// The host is always present in an authority.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Authority;
	///
	/// let authority = Authority::new("example.org:8080").unwrap();
	/// assert_eq!(authority.host(), "example.org");
	/// ```
	pub fn host(&self) -> &Host {
		let bytes = self.as_bytes();
		let range = crate::common::parse::find_host(bytes, 0);
		unsafe { Host::new_unchecked_from_bytes(&bytes[range]) }
	}

	/// Returns a reference to the port, if any.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Authority;
	///
	/// let with_port = Authority::new("example.org:8080").unwrap();
	/// assert_eq!(with_port.port().unwrap(), "8080");
	///
	/// let without = Authority::new("example.org").unwrap();
	/// assert!(without.port().is_none());
	/// ```
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

/// Individual components of a URI authority.
///
/// Contains references to the userinfo, host, and port components.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityParts<'a> {
	/// User information component, if present (e.g., `user:pass`).
	pub user_info: Option<&'a UserInfo>,

	/// Host component (always present).
	pub host: &'a Host,

	/// Port component, if present (e.g., `8080`).
	pub port: Option<&'a Port>,
}

#[cfg(feature = "std")]
impl AuthorityBuf {
	/// Returns a mutable reference to this authority.
	pub fn as_authority_mut(&mut self) -> AuthorityMut<'_> {
		let len = self.0.len();
		unsafe { AuthorityMut::new_unchecked(self.0.as_mut_vec(), 0..len) }
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
		self.as_authority_mut().replace(other);
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
		self.as_authority_mut().try_replace(other)?;
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
		self.as_authority_mut().set_user_info(user_info);
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
		self.as_authority_mut().try_set_user_info(user_info)?;
		Ok(self)
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
		self.as_authority_mut().set_host(host);
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
		self.as_authority_mut().try_set_host(host)?;
		Ok(self)
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
		self.as_authority_mut().set_port(port);
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
		self.as_authority_mut().set_port_u32(port);
		self
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
