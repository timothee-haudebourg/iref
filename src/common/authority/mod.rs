mod r#mut;
pub(crate) use r#mut::*;

macro_rules! authority {
	() => {
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::grammar::Authority)]
		#[newtype(ord([u8], &[u8], Vec<u8>, str, &str, String), owned(AuthorityBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct Authority(str);

		impl Authority {
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
					port: port.map(|r| unsafe {
						super::Port::new_unchecked_from_bytes(&self.0.as_bytes()[r])
					}),
				}
			}

			pub fn user_info(&self) -> Option<&UserInfo> {
				let bytes = self.as_bytes();
				crate::common::parse::find_user_info(bytes, 0)
					.map(|range| unsafe { UserInfo::new_unchecked_from_bytes(&bytes[range]) })
			}

			pub fn host(&self) -> &Host {
				let bytes = self.as_bytes();
				let range = crate::common::parse::find_host(bytes, 0);
				unsafe { Host::new_unchecked_from_bytes(&bytes[range]) }
			}

			pub fn port(&self) -> Option<&super::Port> {
				let bytes = self.as_bytes();
				crate::common::parse::find_port(bytes, 0)
					.map(|range| unsafe { super::Port::new_unchecked_from_bytes(&bytes[range]) })
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
			pub port: Option<&'a super::Port>,
		}
	};
}

pub(crate) use authority;

macro_rules! user_info {
	($name:literal) => {
		#[doc = $name]
		/// authority user info.
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::super::grammar::UserInfo)]
		#[newtype(
			no_deref,
			ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
			owned(
				UserInfoBuf,
				derive(PartialEq, Eq, PartialOrd, Ord, Hash)
			)
		)]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct UserInfo(str);

		impl UserInfo {
			/// Returns the host as a percent-encoded string slice.
			#[inline]
			pub fn as_pct_str(&self) -> &pct_str::PctStr {
				unsafe { pct_str::PctStr::new_unchecked(self.as_str()) }
			}
		}

		impl ::core::ops::Deref for UserInfo {
			type Target = pct_str::PctStr;

			fn deref(&self) -> &Self::Target {
				self.as_pct_str()
			}
		}

		impl ::core::cmp::PartialEq for UserInfo {
			#[inline]
			fn eq(&self, other: &UserInfo) -> bool {
				self.as_pct_str() == other.as_pct_str()
			}
		}

		impl Eq for UserInfo {}

		impl PartialOrd for UserInfo {
			#[inline]
			fn partial_cmp(&self, other: &UserInfo) -> Option<::core::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl Ord for UserInfo {
			#[inline]
			fn cmp(&self, other: &UserInfo) -> ::core::cmp::Ordering {
				self.as_pct_str().cmp(other.as_pct_str())
			}
		}

		impl ::core::hash::Hash for UserInfo {
			#[inline]
			fn hash<H: ::core::hash::Hasher>(&self, hasher: &mut H) {
				self.as_pct_str().hash(hasher)
			}
		}

		impl UserInfoBuf {
			pub fn into_pct_string(self) -> pct_str::PctString {
				unsafe { pct_str::PctString::new_unchecked(self.0) }
			}
		}
	};
}

pub(crate) use user_info;

macro_rules! host {
	($name:literal) => {
		#[doc = $name]
		/// authority host.
		#[derive(static_automata::Validate, str_newtype::StrNewType)]
		#[automaton(super::super::grammar::Host)]
		#[newtype(
			no_deref,
			ord([u8], &[u8], Vec<u8>, str, &str, String, pct_str::PctStr, &pct_str::PctStr, pct_str::PctString),
			owned(HostBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash))
		)]
		#[cfg_attr(feature = "serde", newtype(serde))]
		pub struct Host(str);

		impl Host {
			/// Returns the host as a percent-encoded string slice.
			#[inline]
			pub fn as_pct_str(&self) -> &pct_str::PctStr {
				unsafe { pct_str::PctStr::new_unchecked(self.as_str()) }
			}
		}

		impl core::ops::Deref for Host {
			type Target = pct_str::PctStr;

			fn deref(&self) -> &Self::Target {
				self.as_pct_str()
			}
		}

		impl core::cmp::PartialEq for Host {
			#[inline]
			fn eq(&self, other: &Host) -> bool {
				self.as_pct_str() == other.as_pct_str()
			}
		}

		impl Eq for Host {}

		impl PartialOrd for Host {
			#[inline]
			fn partial_cmp(&self, other: &Host) -> Option<core::cmp::Ordering> {
				Some(self.cmp(other))
			}
		}

		impl Ord for Host {
			#[inline]
			fn cmp(&self, other: &Host) -> core::cmp::Ordering {
				self.as_pct_str().cmp(other.as_pct_str())
			}
		}

		impl core::hash::Hash for Host {
			#[inline]
			fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
				self.as_pct_str().hash(hasher)
			}
		}

		impl HostBuf {
			pub fn into_pct_string(self) -> pct_str::PctString {
				unsafe { pct_str::PctString::new_unchecked(self.0) }
			}
		}
	};
}

pub(crate) use host;
