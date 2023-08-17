use std::ops::Deref;

use crate::common::authority_mut::AuthorityMutImpl;

use super::{Authority, Host, Port, UserInfo};

pub struct AuthorityMut<'a>(AuthorityMutImpl<'a, Authority>);

impl<'a> Deref for AuthorityMut<'a> {
	type Target = Authority;

	fn deref(&self) -> &Self::Target {
		self.as_authority()
	}
}

impl<'a> AuthorityMut<'a> {
	/// Creates a new mutable reference to an URI authority.
	///
	/// # Safety
	///
	/// The `buffer` content inside the range `start..end` must be a valid
	/// URI authority.
	pub unsafe fn new(buffer: &'a mut Vec<u8>, start: usize, end: usize) -> Self {
		Self(AuthorityMutImpl::new(buffer, start, end))
	}

	pub(crate) fn from_impl(i: AuthorityMutImpl<'a, Authority>) -> Self {
		Self(i)
	}

	#[inline]
	pub fn as_authority(&self) -> &Authority {
		self.0.as_authority()
	}

	#[inline]
	pub fn into_authority(self) -> &'a Authority {
		self.0.into_authority()
	}

	#[inline]
	pub fn set_userinfo(&mut self, userinfo: Option<&UserInfo>) {
		self.0.set_userinfo(userinfo)
	}

	#[inline]
	pub fn set_host(&mut self, host: &Host) {
		self.0.set_host(host)
	}

	#[inline]
	pub fn set_port(&mut self, port: Option<&Port>) {
		self.0.set_port(port)
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
