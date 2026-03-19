use core::{
	cmp::Ordering,
	fmt::Write,
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

		let Some(i) = parse_dec_octet(bytes, 0) else { return false };
		if bytes.get(i) != Some(&b'.') { return false }

		let Some(j) = parse_dec_octet(bytes, i + 1) else { return false };
		if bytes.get(j) != Some(&b'.') { return false }

		let Some(k) = parse_dec_octet(bytes, j + 1) else { return false };
		if bytes.get(k) != Some(&b'.') { return false }

		let Some(l) = parse_dec_octet(bytes, k + 1) else { return false };
		l == bytes.len()
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
			result = result << 8 | octet_str.parse::<u8>().unwrap() as u32;
		}

		Some(result)
	}

	/// Parses this host as an IPv6 address and returns it as a `u128`.
	///
	/// Returns `None` if the host is not an IPv6 address.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::Host;
	///
	/// assert_eq!(Host::new("[::1]").unwrap().to_ipv6(), Some(1));
	/// assert_eq!(
	///     Host::new("[2001:db8::1]").unwrap().to_ipv6(),
	///     Some(0x20010db8_00000000_00000000_00000001)
	/// );
	/// assert_eq!(Host::new("[::0]").unwrap().to_ipv6(), Some(0));
	/// assert_eq!(Host::new("127.0.0.1").unwrap().to_ipv6(), None);
	/// ```
	pub fn to_ipv6(&self) -> Option<u128> {
		if !self.is_ipv6() {
			return None;
		}

		let inner = &self.as_str()[1..self.as_str().len() - 1];

		let (left, right) = match inner.split_once("::") {
			Some((l, r)) => (l, Some(r)),
			None => (inner, None),
		};

		let mut result: u128 = 0;
		let mut count: u32 = 0;

		if !left.is_empty() {
			for g in left.split(':') {
				result = result << 16 | u16::from_str_radix(g, 16).unwrap() as u128;
				count += 1;
			}
		}

		if let Some(right) = right {
			let mut right_result: u128 = 0;

			if !right.is_empty() {
				for g in right.split(':') {
					right_result = right_result << 16 | u16::from_str_radix(g, 16).unwrap() as u128;
				}
			}

			result = result.checked_shl((8 - count) * 16).unwrap_or(0) | right_result;
		}

		Some(result)
	}
}

/// Parses a `dec-octet` (RFC 3986) starting at position `i` in `bytes`.
///
/// ```text
/// dec-octet = DIGIT               ; 0-9
///           / %x31-39 DIGIT       ; 10-99
///           / "1" 2DIGIT          ; 100-199
///           / "2" %x30-34 DIGIT   ; 200-249
///           / "25" %x30-35        ; 250-255
/// ```
///
/// Returns `Some(end)` where `end` is the index past the last byte of the
/// octet, or `None` if no valid dec-octet starts at `i`.
fn parse_dec_octet(bytes: &[u8], i: usize) -> Option<usize> {
	let d0 = *bytes.get(i)?;
	if !d0.is_ascii_digit() {
		return None;
	}

	if d0 == b'0' {
		return Some(i + 1);
	}

	// d0 is 1-9
	let Some(&d1) = bytes.get(i + 1) else {
		return Some(i + 1);
	};
	if !d1.is_ascii_digit() {
		return Some(i + 1);
	}

	// d0 is 1-9, d1 is 0-9
	let Some(&d2) = bytes.get(i + 2) else {
		return Some(i + 2);
	};
	if !d2.is_ascii_digit() {
		return Some(i + 2);
	}

	// d0 d1 d2, all digits. Valid only if <= 255.
	if d0 == b'1'
		|| (d0 == b'2' && (d1 <= b'4' || (d1 == b'5' && d2 <= b'5')))
	{
		Some(i + 3)
	} else {
		None
	}
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

	/// Creates a [`HostBuf`] from an IPv4 address represented as a `u32`.
	///
	/// The octets are extracted from the most significant byte first
	/// (e.g., `0x7f000001` becomes `"127.0.0.1"`).
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::HostBuf;
	///
	/// assert_eq!(HostBuf::from_ipv4(0x7f000001).as_str(), "127.0.0.1");
	/// assert_eq!(HostBuf::from_ipv4(0).as_str(), "0.0.0.0");
	/// ```
	pub fn from_ipv4(addr: u32) -> Self {
		let s = format!(
			"{}.{}.{}.{}",
			(addr >> 24) & 0xff,
			(addr >> 16) & 0xff,
			(addr >> 8) & 0xff,
			addr & 0xff
		);
		unsafe { Self::new_unchecked(s) }
	}

	/// Creates a [`HostBuf`] from an IPv6 address represented as a `u128`.
	///
	/// The address is formatted with `::` compression for the longest run
	/// of consecutive zero groups, enclosed in brackets as required by
	/// RFC 3986.
	///
	/// # Example
	///
	/// ```rust
	/// use iref::uri::HostBuf;
	///
	/// assert_eq!(HostBuf::from_ipv6(1).as_str(), "[::1]");
	/// assert_eq!(HostBuf::from_ipv6(0).as_str(), "[::]");
	/// ```
	pub fn from_ipv6(addr: u128) -> Self {
		let s = format!("[{}]", format_ipv6(addr));
		unsafe { Self::new_unchecked(s) }
	}
}

/// Formats an IPv6 address as a string with `::` compression for the
/// longest run of consecutive zero groups.
#[cfg(feature = "std")]
fn format_ipv6(addr: u128) -> String {
	let groups: [u16; 8] = core::array::from_fn(|i| (addr >> (112 - i * 16)) as u16);

	// Find the longest run of consecutive zero groups.
	let mut best_start = 0;
	let mut best_len = 0;
	let mut cur_start = 0;
	let mut cur_len = 0;
	for (i, &g) in groups.iter().enumerate() {
		if g == 0 {
			if cur_len == 0 {
				cur_start = i;
			}
			cur_len += 1;
			if cur_len > best_len {
				best_start = cur_start;
				best_len = cur_len;
			}
		} else {
			cur_len = 0;
		}
	}

	let mut s = String::new();

	let write_groups = |s: &mut String, groups: &[u16]| {
		for (i, g) in groups.iter().enumerate() {
			if i > 0 {
				s.push(':');
			}
			write!(s, "{:x}", g).unwrap();
		}
	};

	if best_len >= 2 {
		write_groups(&mut s, &groups[..best_start]);
		s.push_str("::");
		write_groups(&mut s, &groups[best_start + best_len..]);
	} else {
		write_groups(&mut s, &groups);
	}

	s
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_ipv4_valid() {
		for input in ["0.0.0.0", "255.255.255.255", "127.0.0.1", "1.2.3.4", "249.249.249.249"] {
			assert!(Host::new(input).unwrap().is_ipv4(), "is_ipv4({input}) should be true");
		}
	}

	#[test]
	fn is_ipv4_invalid() {
		for input in ["example.org", "[::1]", "1.2.3", "1.2.3.4.5", "999.0.0.1", "256.0.0.1"] {
			assert!(!Host::new(input).unwrap().is_ipv4(), "is_ipv4({input}) should be false");
		}
	}

	#[test]
	fn to_ipv4() {
		let vectors: [(&str, Option<u32>); _] = [
			("0.0.0.0", Some(0)),
			("255.255.255.255", Some(0xFFFFFFFF)),
			("127.0.0.1", Some(0x7F000001)),
			("192.168.1.1", Some(0xC0A80101)),
			("10.0.0.1", Some(0x0A000001)),
			("1.2.3.4", Some(0x01020304)),
			// Not IPv4
			("[::1]", None),
			("example.org", None),
		];

		for (input, expected) in vectors {
			let host = Host::new(input).unwrap();
			assert_eq!(host.to_ipv4(), expected, "to_ipv4({input})");
		}
	}

	#[test]
	fn to_ipv6() {
		let vectors: [(&str, Option<u128>); _] = [
			// All zeros
			("[::]", Some(0)),
			// All ones
			("[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]", Some(u128::MAX)),
			// Loopback
			("[::1]", Some(1)),
			// Full address, no compression
			(
				"[2001:0db8:85a3:0000:0000:8a2e:0370:7334]",
				Some(0x20010db885a3000000008a2e03707334),
			),
			// Compression in the middle
			("[2001:db8::1]", Some(0x20010db8_00000000_00000000_00000001)),
			// Right-only expansion
			("[1:2:3::]", Some(0x00010002000300000000000000000000)),
			// Left-only expansion
			("[::1:2:3]", Some(0x00000000000000000000000100020003)),
			// Single group
			("[::ffff]", Some(0xffff)),
			// Not IPv6
			("127.0.0.1", None),
			("example.org", None),
		];

		for (input, expected) in vectors {
			let host = Host::new(input).unwrap();
			assert_eq!(host.to_ipv6(), expected, "to_ipv6({input})");
		}
	}

	#[test]
	fn from_ipv4_round_trip() {
		let vectors: [(u32, &str); _] = [
			(0, "0.0.0.0"),
			(0xFFFFFFFF, "255.255.255.255"),
			(0x7F000001, "127.0.0.1"),
			(0xC0A80101, "192.168.1.1"),
			(0x0A000001, "10.0.0.1"),
		];

		for (addr, expected_str) in vectors {
			let host = HostBuf::from_ipv4(addr);
			assert_eq!(host.as_str(), expected_str, "from_ipv4(0x{addr:08x})");
			assert!(host.is_ipv4());
			assert_eq!(host.to_ipv4(), Some(addr));
		}
	}

	#[test]
	fn from_ipv6_round_trip() {
		let vectors: [(u128, &str); _] = [
			(0, "[::]"),
			(1, "[::1]"),
			(u128::MAX, "[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]"),
			(0x20010db8_00000000_00000000_00000001, "[2001:db8::1]"),
			// No compression for a single zero group
			(0x00010000000300040005000600070008, "[1:0:3:4:5:6:7:8]"),
			// Compression picks the longest run
			(0x00010002000000000000000000070008, "[1:2::7:8]"),
			// Compression picks the first longest run on tie
			(0x00010000000000040005000000000008, "[1::4:5:0:0:8]"),
		];

		for (addr, expected_str) in vectors {
			let host = HostBuf::from_ipv6(addr);
			assert_eq!(host.as_str(), expected_str, "from_ipv6(0x{addr:032x})");
			assert!(host.is_ipv6());
			assert_eq!(host.to_ipv6(), Some(addr));
		}
	}
}
