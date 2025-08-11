use core::ops::Range;

use crate::uri::Port;

use super::parse::{self, UserInfoOrHost};

pub struct AuthorityPartsImpl {
	pub user_info: Option<Range<usize>>,
	pub host: Range<usize>,
	pub port: Option<Range<usize>>,
}

pub trait AuthorityImpl: PartialEq {
	type UserInfo: ?Sized + UserInofImpl;
	type Host: ?Sized + HostImpl;

	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	fn as_bytes(&self) -> &[u8];

	fn len(&self) -> usize {
		self.as_bytes().len()
	}

	fn parts(&self) -> AuthorityPartsImpl {
		let bytes = self.as_bytes();
		let (user_info, host) = match parse::user_info_or_host(bytes, 0) {
			(UserInfoOrHost::UserInfo, user_info_end) => {
				let host_start = user_info_end + 1;
				let host_end = parse::host(bytes, host_start);
				(Some(0..user_info_end), host_start..host_end)
			}
			(UserInfoOrHost::Host, host_end) => (None, 0..host_end),
		};

		let (has_port, port_end) = parse::port(bytes, host.end);
		let port = has_port.then_some((host.end + 1)..port_end);

		AuthorityPartsImpl {
			user_info,
			host,
			port,
		}
	}

	fn user_info(&self) -> Option<&Self::UserInfo> {
		let bytes = self.as_bytes();
		parse::find_user_info(bytes, 0)
			.map(|range| unsafe { Self::UserInfo::new_unchecked(&bytes[range]) })
	}

	fn host(&self) -> &Self::Host {
		let bytes = self.as_bytes();
		let range = parse::find_host(bytes, 0);
		unsafe { Self::Host::new_unchecked(&bytes[range]) }
	}

	fn port(&self) -> Option<&Port> {
		let bytes = self.as_bytes();
		parse::find_port(bytes, 0).map(|range| unsafe { Port::new_unchecked(&bytes[range]) })
	}
}

pub trait UserInofImpl {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	fn as_bytes(&self) -> &[u8];

	fn len(&self) -> usize {
		self.as_bytes().len()
	}
}

pub trait HostImpl {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	fn as_bytes(&self) -> &[u8];

	fn len(&self) -> usize {
		self.as_bytes().len()
	}
}
