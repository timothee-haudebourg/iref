use std::{marker::PhantomData, ops::Range};

use super::{
	authority::{AuthorityImpl, HostImpl, UserInofImpl},
	parse,
};
use crate::uri::Port;

pub struct AuthorityMutImpl<'a, A: ?Sized> {
	/// The whole URI/IRI (reference) data.
	data: &'a mut Vec<u8>,

	start: usize,

	end: usize,

	a: PhantomData<A>,
}

impl<'a, A: ?Sized + AuthorityImpl> AuthorityMutImpl<'a, A> {
	pub unsafe fn new(data: &'a mut Vec<u8>, start: usize, end: usize) -> Self {
		Self {
			data,
			start,
			end,
			a: PhantomData,
		}
	}

	#[inline]
	fn replace(&mut self, range: Range<usize>, content: &[u8]) {
		crate::utils::replace(self.data, range, content)
	}

	#[inline]
	fn allocate(&mut self, range: Range<usize>, len: usize) {
		crate::utils::allocate_range(self.data, range, len)
	}

	pub fn as_authority(&self) -> &A {
		unsafe { A::new_unchecked(&self.data[self.start..self.end]) }
	}

	pub fn into_authority(self) -> &'a A {
		unsafe { A::new_unchecked(&self.data[self.start..self.end]) }
	}

	#[inline]
	pub fn set_userinfo(&mut self, userinfo: Option<&A::UserInfo>) {
		let bytes = &self.data[..self.end];

		match userinfo {
			Some(new_userinfo) => match parse::find_user_info(bytes, self.start) {
				Some(userinfo_range) => self.replace(userinfo_range, new_userinfo.as_bytes()),
				None => {
					let added_len = new_userinfo.len() + 1;
					self.allocate(self.start..self.start, added_len);
					self.data[self.start..(self.start + new_userinfo.len())]
						.copy_from_slice(new_userinfo.as_bytes());
					self.data[self.start + new_userinfo.len()] = b'@';
					self.end += added_len
				}
			},
			None => {
				if let Some(userinfo_range) = parse::find_user_info(bytes, self.start) {
					self.replace(userinfo_range.start..(userinfo_range.end + 1), b"");
					self.end -= userinfo_range.end - userinfo_range.start;
				}
			}
		}
	}

	#[inline]
	pub fn set_host(&mut self, host: &A::Host) {
		let bytes = &self.data[..self.end];
		let range = parse::find_host(bytes, self.start);
		let host_len = range.end - range.start;

		if host_len > host.len() {
			self.end -= host_len - host.len()
		} else {
			self.end -= host.len() - host_len
		}

		self.replace(range, host.as_bytes());
	}

	#[inline]
	pub fn set_port(&mut self, port: Option<&Port>) {
		let bytes = &self.data[..self.end];
		match port {
			Some(new_port) => match parse::find_port(bytes, self.start) {
				Some(range) => self.replace(range, new_port.as_bytes()),
				None => {
					let added_len = new_port.len() + 1;
					self.allocate(self.end..self.end, added_len);
					self.data[self.end] = b':';
					self.data[(self.end + 1)..(self.end + added_len)]
						.copy_from_slice(new_port.as_bytes());
					self.end += added_len;
				}
			},
			None => {
				if let Some(port_range) = parse::find_port(bytes, self.start) {
					self.replace((port_range.start - 1)..port_range.end, b"");
					self.end -= port_range.end - port_range.start;
				}
			}
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
