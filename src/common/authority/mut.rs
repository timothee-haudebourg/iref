macro_rules! authority_mut {
	() => {
		pub struct AuthorityMut<'a> {
			/// The whole URI/IRI (reference) data.
			data: &'a mut Vec<u8>,

			start: usize,

			end: usize,
		}

		impl<'a> AuthorityMut<'a> {
			/// Creates a new mutable authority part.
			///
			/// # Safety
			///
			/// The buffer content between the range `start..end` must be a valid
			/// authority.
			pub unsafe fn new(data: &'a mut Vec<u8>, start: usize, end: usize) -> Self {
				Self { data, start, end }
			}

			fn replace_bytes(&mut self, range: core::ops::Range<usize>, content: &[u8]) {
				crate::utils::replace(self.data, range, content)
			}

			fn allocate_bytes(&mut self, range: core::ops::Range<usize>, len: usize) {
				crate::utils::allocate_range(self.data, range, len)
			}

			pub fn as_authority(&self) -> &super::Authority {
				unsafe {
					super::Authority::new_unchecked_from_bytes(&self.data[self.start..self.end])
				}
			}

			pub fn into_authority(self) -> &'a super::Authority {
				unsafe {
					super::Authority::new_unchecked_from_bytes(&self.data[self.start..self.end])
				}
			}

			#[inline]
			pub fn set_userinfo(&mut self, userinfo: Option<&super::UserInfo>) {
				let bytes = &self.data[..self.end];

				match userinfo {
					Some(new_userinfo) => {
						match crate::common::parse::find_user_info(bytes, self.start) {
							Some(userinfo_range) => {
								self.replace_bytes(userinfo_range, new_userinfo.as_bytes())
							}
							None => {
								let added_len = new_userinfo.len() + 1;
								self.allocate_bytes(self.start..self.start, added_len);
								self.data[self.start..(self.start + new_userinfo.len())]
									.copy_from_slice(new_userinfo.as_bytes());
								self.data[self.start + new_userinfo.len()] = b'@';
								self.end += added_len
							}
						}
					}
					None => {
						if let Some(userinfo_range) =
							crate::common::parse::find_user_info(bytes, self.start)
						{
							self.replace_bytes(userinfo_range.start..(userinfo_range.end + 1), b"");
							self.end -= userinfo_range.end - userinfo_range.start;
						}
					}
				}
			}

			#[inline]
			pub fn set_host(&mut self, host: &super::Host) {
				let bytes = &self.data[..self.end];
				let range = crate::common::parse::find_host(bytes, self.start);
				let host_len = range.end - range.start;

				if host_len > host.len() {
					self.end -= host_len - host.len()
				} else {
					self.end -= host.len() - host_len
				}

				self.replace_bytes(range, host.as_bytes());
			}

			#[inline]
			pub fn try_set_host<'s>(&mut self, host: &'s str) -> Result<(), InvalidHost<&'s str>> {
				self.set_host(host.try_into()?);
				Ok(())
			}

			#[inline]
			pub fn set_port(&mut self, port: Option<&crate::uri::Port>) {
				let bytes = &self.data[..self.end];
				match port {
					Some(new_port) => match crate::common::parse::find_port(bytes, self.start) {
						Some(range) => self.replace_bytes(range, new_port.as_bytes()),
						None => {
							let added_len = new_port.len() + 1;
							self.allocate_bytes(self.end..self.end, added_len);
							self.data[self.end] = b':';
							self.data[(self.end + 1)..(self.end + added_len)]
								.copy_from_slice(new_port.as_bytes());
							self.end += added_len;
						}
					},
					None => {
						if let Some(port_range) = crate::common::parse::find_port(bytes, self.start)
						{
							self.replace_bytes((port_range.start - 1)..port_range.end, b"");
							self.end -= port_range.end - port_range.start;
						}
					}
				}
			}
		}

		impl<'a> ::core::ops::Deref for AuthorityMut<'a> {
			type Target = super::Authority;

			fn deref(&self) -> &Self::Target {
				self.as_authority()
			}
		}
	};
}

pub(crate) use authority_mut;

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
