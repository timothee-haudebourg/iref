use core::{
	cmp::Ordering,
	hash::{Hash, Hasher},
	ops::Deref,
};

use pct_str::PctStr;

/// URI authority host.
#[derive(static_automata::Validate, str_newtype::StrNewType)]
#[automaton(crate::uri::grammar::Host)]
#[newtype(
	no_deref,
	ord([u8], &[u8], str, &str, pct_str::PctStr, &pct_str::PctStr)
)]
#[cfg_attr(
	feature = "std",
	newtype(ord(Vec<u8>, String, pct_str::PctString), owned(HostBuf, derive(PartialEq, Eq, PartialOrd, Ord, Hash)))
)]
#[cfg_attr(feature = "serde", newtype(serde))]
pub struct Host(str);

impl Host {
	/// Returns the host as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	/// Returns `true` if this host is an IP-literal (IPv6 address or
	/// IPvFuture).
	///
	/// IP-literals are enclosed in brackets (`[...]`) as defined in
	/// RFC 3986.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Host;
	///
	/// assert!(Host::new("[::1]").unwrap().is_ip_literal());
	/// assert!(!Host::new("example.org").unwrap().is_ip_literal());
	/// ```
	pub fn is_ip_literal(&self) -> bool {
		self.as_bytes().first() == Some(&b'[')
	}

	/// Returns `true` if this host is an IPv4 address.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Host;
	///
	/// assert!(Host::new("127.0.0.1").unwrap().is_ipv4());
	/// assert!(!Host::new("[::1]").unwrap().is_ipv4());
	/// assert!(!Host::new("example.org").unwrap().is_ipv4());
	/// ```
	pub fn is_ipv4(&self) -> bool {
		let bytes = self.as_bytes();
		!bytes.is_empty()
			&& bytes[0].is_ascii_digit()
			&& bytes.iter().all(|&b| b.is_ascii_digit() || b == b'.')
			&& bytes.iter().filter(|&&b| b == b'.').count() == 3
	}

	/// Returns `true` if this host is an IPv6 address.
	///
	/// IPv6 addresses are enclosed in brackets (`[...]`) and do not start
	/// with `[v` (which denotes IPvFuture).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Host;
	///
	/// assert!(Host::new("[::1]").unwrap().is_ipv6());
	/// assert!(Host::new("[2001:db8::1]").unwrap().is_ipv6());
	/// assert!(!Host::new("127.0.0.1").unwrap().is_ipv6());
	/// assert!(!Host::new("example.org").unwrap().is_ipv6());
	/// ```
	pub fn is_ipv6(&self) -> bool {
		let bytes = self.as_bytes();
		bytes.len() >= 4 && bytes[0] == b'[' && bytes[1] != b'v'
	}

	/// Parses this host as an IPv4 address and returns it as a `u32`.
	///
	/// Returns `None` if the host is not a valid IPv4 address.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Host;
	///
	/// assert_eq!(Host::new("127.0.0.1").unwrap().to_ipv4(), Some(0x7f000001));
	/// assert_eq!(Host::new("0.0.0.0").unwrap().to_ipv4(), Some(0));
	/// assert_eq!(Host::new("[::1]").unwrap().to_ipv4(), None);
	/// ```
	pub fn to_ipv4(&self) -> Option<u32> {
		if !self.is_ipv4() {
			return None;
		}

		let mut result: u32 = 0;
		for octet_str in self.as_str().split('.') {
			result = result << 8 | parse_ipv4_octet(octet_str.as_bytes())? as u32;
		}

		Some(result)
	}
}

fn parse_ipv4_octet(bytes: &[u8]) -> Option<u8> {
	if bytes.is_empty() {
		return None;
	}

	let mut value: u16 = 0;
	for &b in bytes {
		value = value * 10 + (b - b'0') as u16;
	}

	u8::try_from(value).ok()
}

impl Deref for Host {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl PartialEq for Host {
	#[inline]
	fn eq(&self, other: &Host) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Host {}

impl PartialOrd for Host {
	#[inline]
	fn partial_cmp(&self, other: &Host) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Host {
	#[inline]
	fn cmp(&self, other: &Host) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Host {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

#[cfg(feature = "std")]
impl HostBuf {
	pub fn into_pct_string(self) -> pct_str::PctString {
		unsafe { pct_str::PctString::new_unchecked(self.0) }
	}
}

/// Parses a URI authority [`Host`] at compile time.
#[macro_export]
macro_rules! host {
	($value:literal) => {
		match $crate::uri::Host::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI authority host"),
		}
	};
}
