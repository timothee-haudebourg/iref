mod utf8;

use super::Error;

#[derive(Default, Clone, Copy)]
pub struct ParsedAuthority {
	pub userinfo_len: Option<usize>,
	pub host_len: usize,
	pub port_len: Option<usize>,
}

impl ParsedAuthority {
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.userinfo_len.is_none() && self.host_len == 0 && self.port_len.is_none()
	}

	#[inline]
	pub fn len(&self) -> usize {
		let mut len = 0;

		if let Some(l) = self.userinfo_len {
			len += l + 1;
		}

		len += self.host_len;

		if let Some(l) = self.port_len {
			len += 1 + l;
		}

		len
	}

	#[inline]
	pub fn host_offset(&self) -> usize {
		let mut offset = 0;

		if let Some(l) = self.userinfo_len {
			offset += l + 1;
		}

		offset
	}

	#[inline]
	pub fn port_offset(&self) -> usize {
		let mut offset = 0;

		if let Some(l) = self.userinfo_len {
			offset += l + 1;
		}

		offset += self.host_len;

		if self.port_len.is_some() {
			offset += 1;
		}

		offset
	}
}

#[derive(Default, Clone, Copy)]
pub struct ParsedIriRef {
	pub scheme_len: Option<usize>,
	pub authority: Option<ParsedAuthority>,
	pub path_len: usize,
	pub query_len: Option<usize>,
	pub fragment_len: Option<usize>,
}

impl ParsedIriRef {
	#[inline]
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<ParsedIriRef, Error> {
		let buffer = buffer.as_ref();
		let mut scheme_len = None;
		let mut authority = None;
		let path_len;
		let mut query_len = None;
		let mut fragment_len = None;

		let scheme_len_tmp = parse_scheme(buffer, 0)?;
		let scheme_end = if let Some((':', 1)) = get_char(buffer, scheme_len_tmp)? {
			if scheme_len_tmp == 0 {
				return Err(Error::MissingScheme);
			}

			scheme_len = Some(scheme_len_tmp);
			scheme_len_tmp + 1
		} else {
			0
		};

		let authority_end;

		match get_char(buffer, scheme_end)? {
			Some(('/', 1)) => {
				match get_char(buffer, scheme_end + 1)? {
					Some(('/', 1)) => {
						authority = Some(parse_authority(buffer, scheme_end + 2)?);
						authority_end = scheme_end + 2 + authority.unwrap().len();
						// path must be absolute.
						path_len = if let Some(('/', 1)) = get_char(buffer, authority_end)? {
							parse_path(buffer, authority_end)?
						} else {
							0
						};
					}
					_ => {
						authority_end = scheme_end;
						path_len = parse_path(buffer, authority_end)?;
					}
				}
			}
			_ => {
				authority_end = scheme_end;
				path_len = parse_path(buffer, authority_end)?;
			}
		}

		let i = authority_end + path_len;

		match get_char(buffer, i)? {
			Some(('#', 1)) => fragment_len = Some(parse_fragment(buffer, i + 1)?),
			Some(('?', 1)) => {
				let len = parse_query(buffer, i + 1)?;
				query_len = Some(len);
				match get_char(buffer, i + 1 + len)? {
					Some(('#', 1)) => fragment_len = Some(parse_fragment(buffer, i + 1 + len + 1)?),
					Some(_) => return Err(Error::InvalidPath),
					None => (),
				}
			}
			Some(_) => return Err(Error::InvalidPath),
			None => (),
		}

		let data = ParsedIriRef {
			scheme_len,
			authority,
			path_len,
			query_len,
			fragment_len,
		};

		if data.len() == buffer.len() {
			Ok(data)
		} else if fragment_len.is_some() {
			Err(Error::InvalidFragment)
		} else if query_len.is_some() {
			Err(Error::InvalidQuery)
		} else {
			Err(Error::InvalidPath)
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		let mut offset = 0;

		if let Some(len) = self.scheme_len {
			offset += len + 1;
		}

		if let Some(authority) = self.authority.as_ref() {
			offset += 2 + authority.len();
		}

		offset += self.path_len;

		if let Some(len) = self.query_len {
			offset += 1 + len;
		}

		if let Some(len) = self.fragment_len {
			offset += 1 + len;
		}

		offset
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.scheme_len.is_none()
			&& self.authority.is_none()
			&& self.path_len == 0
			&& self.query_len.is_none()
			&& self.fragment_len.is_none()
	}

	#[inline]
	pub fn authority_offset(&self) -> usize {
		let mut offset = 0;

		if let Some(len) = self.scheme_len {
			offset += len + 1;
		}

		if self.authority.is_some() {
			offset += 2;
		}

		offset
	}

	#[inline]
	pub fn path_offset(&self) -> usize {
		let mut offset = 0;

		if let Some(len) = self.scheme_len {
			offset += len + 1;
		}

		if let Some(authority) = self.authority.as_ref() {
			offset += 2 + authority.len();
		}

		offset
	}

	#[inline]
	pub fn query_offset(&self) -> usize {
		let mut offset = self.path_offset() + self.path_len;

		if self.query_len.is_some() {
			offset += 1;
		}

		offset
	}

	#[inline]
	pub fn fragment_offset(&self) -> usize {
		let mut offset = self.path_offset() + self.path_len;

		if let Some(len) = self.query_len {
			offset += 1 + len;
		}

		if self.fragment_len.is_some() {
			offset += 1;
		}

		offset
	}
}

#[inline]
pub fn get_char(buffer: &[u8], i: usize) -> Result<Option<(char, usize)>, Error> {
	match utf8::get_char(buffer, i) {
		Ok(None) => Ok(None),
		Ok(Some((c, len))) => Ok(Some((c, len as usize))),
		Err(_) => Err(Error::InvalidEncoding),
	}
}

#[inline]
pub fn is_alpha(c: char) -> bool {
	c.is_ascii_alphabetic()
}

#[inline]
pub fn is_digit(c: char) -> bool {
	c.is_ascii_digit()
}

#[inline]
pub fn is_alphanumeric(c: char) -> bool {
	c.is_ascii_alphanumeric()
}

/// Parse the IRI scheme.
#[inline]
pub fn parse_scheme(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	loop {
		match get_char(buffer, i)? {
			Some((c, len))
				if (i == 0 && is_alpha(c))
					|| (i > 0 && (is_alphanumeric(c) || c == '+' || c == '-' || c == '.')) =>
			{
				i += len
			}
			_ => break,
		}
	}

	Ok(i)
}

fn is_ucschar(c: char) -> bool {
	let c = c as u32;
	(0xA0..=0xD7FF).contains(&c)
		|| (0xF900..=0xFDCF).contains(&c)
		|| (0xFDF0..=0xFFEF).contains(&c)
		|| (0x10000..=0x1FFFD).contains(&c)
		|| (0x20000..=0x2FFFD).contains(&c)
		|| (0x30000..=0x3FFFD).contains(&c)
		|| (0x40000..=0x4FFFD).contains(&c)
		|| (0x50000..=0x5FFFD).contains(&c)
		|| (0x60000..=0x6FFFD).contains(&c)
		|| (0x70000..=0x7FFFD).contains(&c)
		|| (0x80000..=0x8FFFD).contains(&c)
		|| (0x90000..=0x9FFFD).contains(&c)
		|| (0xA0000..=0xAFFFD).contains(&c)
		|| (0xB0000..=0xBFFFD).contains(&c)
		|| (0xC0000..=0xCFFFD).contains(&c)
		|| (0xD0000..=0xDFFFD).contains(&c)
		|| (0xE1000..=0xEFFFD).contains(&c)
}

fn is_private(c: char) -> bool {
	let c = c as u32;
	(0xE000..=0xF8FF).contains(&c)
		|| (0xF0000..=0xFFFFD).contains(&c)
		|| (0x100000..=0x10FFFD).contains(&c)
}

fn is_unreserved(c: char) -> bool {
	is_alphanumeric(c) || c == '-' || c == '.' || c == '_' || c == '~' || is_ucschar(c)
}

fn is_subdelim(c: char) -> bool {
	matches!(
		c,
		'!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
	)
}

fn is_hex_digit(buffer: &[u8], i: usize) -> Result<bool, Error> {
	match get_char(buffer, i)? {
		Some((c, 1)) => Ok(c.is_ascii_hexdigit()),
		_ => Ok(false),
	}
}

fn parse_pct_encoded(buffer: &[u8], i: usize) -> Result<Option<usize>, Error> {
	match get_char(buffer, i)? {
		Some(('%', 1)) => {
			if is_hex_digit(buffer, i + 1)? && is_hex_digit(buffer, i + 2)? {
				Ok(Some(3))
			} else {
				Err(Error::InvalidPercentEncoding)
			}
		}
		_ => Ok(None),
	}
}

#[inline]
pub fn parse_userinfo(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;

	loop {
		match get_char(buffer, i)? {
			// Some(('@', 1)) => {
			// 	return Ok(Some(i))
			// },
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len)) if c == ':' || is_subdelim(c) || is_unreserved(c) => i += len,
			_ => break,
		}
	}

	Ok(i - offset)
}

#[inline]
pub fn parse_query(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;

	loop {
		match get_char(buffer, i)? {
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len))
				if c == ':'
					|| c == '@' || c == '/'
					|| c == '?' || is_subdelim(c)
					|| is_unreserved(c) || is_private(c) =>
			{
				i += len
			}
			_ => break,
		}
	}

	Ok(i - offset)
}

#[inline]
pub fn parse_fragment(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;

	loop {
		match get_char(buffer, i)? {
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len))
				if c == ':'
					|| c == '@' || c == '/'
					|| c == '?' || is_subdelim(c)
					|| is_unreserved(c) =>
			{
				i += len
			}
			_ => break,
		}
	}

	Ok(i - offset)
}

fn parse_dec_octet(buffer: &[u8], i: usize) -> Result<Option<(u32, usize)>, Error> {
	let mut octet = 0u32;
	let mut len = 0;

	while let Some((c, 1)) = get_char(buffer, i + len)? {
		if let Some(d) = c.to_digit(10) {
			if octet == 25 && d > 5 {
				return Ok(None);
			} else {
				octet = octet * 10 + d
			}

			len += 1;

			if len >= 3 || octet > 25 {
				break;
			}
		} else {
			break;
		}
	}

	if len == 0 {
		Ok(None)
	} else {
		Ok(Some((octet, len)))
	}
}

/// Parse an IPv4 literal.
///
/// If `host` is `true` then the successor of a `IPv4address` can be
/// `:`, `/`, `?`, `#` or EOF.
/// Otherwise, the successor can be ']'.
fn parse_ipv4_literal(
	buffer: &[u8],
	mut i: usize,
	host: bool,
) -> Result<Option<(u32, usize)>, Error> {
	let offset = i;
	if let Some((a, olen)) = parse_dec_octet(buffer, i)? {
		i += olen;
		if let Some(('.', 1)) = get_char(buffer, i)? {
			i += 1;
			if let Some((b, olen)) = parse_dec_octet(buffer, i)? {
				i += olen;
				if let Some(('.', 1)) = get_char(buffer, i)? {
					i += 1;
					if let Some((c, olen)) = parse_dec_octet(buffer, i)? {
						i += olen;
						if let Some(('.', 1)) = get_char(buffer, i)? {
							i += 1;
							if let Some((d, olen)) = parse_dec_octet(buffer, i)? {
								i += olen;

								// Check the successor.
								match get_char(buffer, i)? {
									Some((':' | '/' | '?' | '#', 1)) | None if host => (),
									Some((']', 1)) | None if !host => (),
									_ => return Ok(None),
								}

								let ipv4 = (a << 24) | (b << 16) | (c << 8) | d;
								let len = i - offset;
								return Ok(Some((ipv4, len)));
							}
						}
					}
				}
			}
		}
	}

	Ok(None)
}

fn parse_h16(buffer: &[u8], i: usize) -> Result<Option<(u16, usize)>, Error> {
	let mut len = 0;
	let mut h16 = 0;

	loop {
		match get_char(buffer, i + len)? {
			Some((':' | ']', 1)) | None if len > 0 => break Ok(Some((h16, len))),
			Some((c, 1)) if len < 4 => {
				if let Some(d) = c.to_digit(16) {
					h16 = (h16 << 4) | d as u16;
					len += 1;
				} else {
					break Ok(None);
				}
			}
			_ => break Ok(None),
		}
	}
}

/// Parse an IPv6 literal.
/// Return the IPv6 and the string length.
///
/// Followed by `]`.
fn parse_ipv6_literal(buffer: &[u8], mut i: usize) -> Result<Option<(u128, usize)>, Error> {
	let offset = i;

	let mut lhs_shift = 128;
	let mut is_lhs = true;
	let mut lhs = 0u128;
	let mut rhs = 0u128;

	let mut h16_count = 0;

	if let Some((':', 1)) = get_char(buffer, i)? {
		i += 1;
		match get_char(buffer, i)? {
			Some((':', 1)) => (),
			_ => return Ok(None),
		}
	}

	loop {
		match get_char(buffer, i)? {
			Some((':', 1)) if is_lhs => {
				// `::` separator.
				i += 1;
				is_lhs = false;
				h16_count += 1;

				if let Some((']', 1)) | None = get_char(buffer, i)? {
					break Ok(Some((lhs | rhs, i - offset)));
				}
			}
			_ if h16_count < 8 => match parse_h16(buffer, i)? {
				// maybe `h16` group.
				Some((h16, len)) => {
					i += len;
					h16_count += 1;

					if is_lhs {
						lhs_shift -= 16;
						lhs |= (h16 as u128) << lhs_shift;
					} else {
						rhs = (rhs << 16) | h16 as u128
					}

					if (!is_lhs && h16_count <= 8) || h16_count == 8 {
						if let Some((']', 1)) | None = get_char(buffer, i)? {
							break Ok(Some((lhs | rhs, i - offset)));
						}
					}

					match get_char(buffer, i)? {
						Some((':', 1)) if h16_count < 8 => {
							i += 1;
						}
						_ => break Ok(None),
					}
				}
				None => {
					// not a `h16` group.
					if (is_lhs && h16_count == 6) || (!is_lhs && h16_count <= 6) {
						// maybe an IPv4 literal.
						match parse_ipv4_literal(buffer, i, false)? {
							Some((ipv4, len)) => {
								i += len;

								if is_lhs {
									debug_assert_eq!(lhs_shift, 32);
									lhs |= ipv4 as u128;
								} else {
									rhs = (rhs << 32) | ipv4 as u128
								}

								if let Some((']', 1)) | None = get_char(buffer, i)? {
									break Ok(Some((lhs | rhs, i - offset)));
								}
							}
							None => break Ok(None),
						}
					} else {
						break Ok(None);
					}
				}
			},
			_ => break Ok(None),
		}
	}
}

/// Parse an IPvFuture literal.
///
/// ```abnf
/// IPvFuture  = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
/// ```
///
/// Followed by `]`.
/// Returns the string length.
fn parse_ipvfuture_literal(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	let offset = i;
	if let Some(('v', 1)) = get_char(buffer, i)? {
		i += 1;
		if is_hex_digit(buffer, i)? {
			i += 1;
			while is_hex_digit(buffer, i)? {
				i += 1
			}

			if let Some(('.', 1)) = get_char(buffer, i)? {
				i += 1;

				match get_char(buffer, i)? {
					Some((c, len)) if c == ':' || is_unreserved(c) || is_subdelim(c) => {
						i += len;

						loop {
							match get_char(buffer, i)? {
								Some((']', 1)) | None => {
									let len = i - offset;
									return Ok(Some(len));
								}
								Some((c, len))
									if c == ':' || is_unreserved(c) || is_subdelim(c) =>
								{
									i += len;
								}
								_ => break,
							}
						}
					}
					_ => (),
				}
			}
		}
	}

	Ok(None)
}

/// Parse a the `IP-literal` grammar rule:
/// ```abnf
/// IP-literal = "[" ( IPv6address / IPvFuture  ) "]"
/// ```
fn parse_ip_literal(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	let offset = i;
	if let Some(('[', 1)) = get_char(buffer, i)? {
		i += 1;
		if let Some((_, l)) = parse_ipv6_literal(buffer, i)? {
			i += l;
		} else if let Some(l) = parse_ipvfuture_literal(buffer, i)? {
			i += l;
		} else {
			return Ok(None);
		}

		if let Some((']', 1)) = get_char(buffer, i)? {
			i += 1;
			let len = i - offset;
			return Ok(Some(len));
		}
	}

	Ok(None)
}

/// Parse a `reg-name`.
///
/// ```abnf
/// reg-name = *( unreserved / pct-encoded / sub-delims )
/// ```
fn parse_ireg_name(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;
	loop {
		match get_char(buffer, i)? {
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len)) if is_subdelim(c) || is_unreserved(c) => i += len,
			_ => break,
		}
	}

	Ok(i - offset)
}

/// Parse a the `host` grammar rule:
/// ```abnf
/// host = IP-literal / IPv4address / reg-name
/// ```
///
/// The successor of a `host` can be `:`, `/`, `?`, `#` or EOF.
#[inline]
pub fn parse_host(buffer: &[u8], i: usize) -> Result<usize, Error> {
	if let Some(len) = parse_ip_literal(buffer, i)? {
		Ok(len)
	} else if let Some((_, len)) = parse_ipv4_literal(buffer, i, true)? {
		Ok(len)
	} else {
		parse_ireg_name(buffer, i)
	}
}

#[inline]
pub fn parse_port(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;

	while let Some((c, 1)) = get_char(buffer, i)? {
		if c.is_ascii_digit() {
			i += 1
		} else {
			break;
		}
	}

	Ok(i - offset)
}

/// Parse the IRI authority.
#[inline]
pub fn parse_authority(buffer: &[u8], mut i: usize) -> Result<ParsedAuthority, Error> {
	let mut userinfo_len = None;

	let userinfo_tmp_len = parse_userinfo(buffer, i)?;
	if let Some(('@', 1)) = get_char(buffer, i + userinfo_tmp_len)? {
		userinfo_len = Some(userinfo_tmp_len);
		i += userinfo_tmp_len + 1;
	}

	let host_len = parse_host(buffer, i)?;
	i += host_len;

	let port_len = match get_char(buffer, i)? {
		Some((':', 1)) => {
			i += 1;
			Some(parse_port(buffer, i)?)
		}
		_ => None,
	};

	Ok(ParsedAuthority {
		userinfo_len,
		host_len,
		port_len,
	})
}

/// Parse IRI path.
#[inline]
pub fn parse_path(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let start = i;

	loop {
		match get_char(buffer, i)? {
			None | Some(('?', _)) | Some(('#', _)) => break,
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len))
				if is_subdelim(c) || is_unreserved(c) || c == '@' || c == ':' || c == '/' =>
			{
				i += len
			}
			_ => break,
		}
	}

	Ok(i - start)
}

/// Parse IRI path segment.
#[inline]
pub fn parse_path_segment(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let start = i;

	loop {
		match get_char(buffer, i)? {
			None | Some(('?', _)) | Some(('#', _)) | Some(('/', _)) => break,
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break;
				}
			}
			Some((c, len)) if is_subdelim(c) || is_unreserved(c) || c == '@' || c == ':' => {
				i += len
			}
			_ => break,
		}
	}

	Ok(i - start)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ipv6_rule1_1() {
		assert_eq!(
			parse_ipv6_literal(b"2001:0db8:0000:0000:0000:ff00:0042:8329", 0).unwrap(),
			Some((0x2001_0db8_0000_0000_0000_ff00_0042_8329, 39))
		);
	}

	#[test]
	fn ipv6_rule1_2() {
		assert_eq!(
			parse_ipv6_literal(b"2001:0db8:0000:0000:0000:ff00:0.66.131.41", 0).unwrap(),
			Some((0x2001_0db8_0000_0000_0000_ff00_0042_8329, 41))
		);
	}

	#[test]
	fn ipv6_rule2_1() {
		assert_eq!(
			parse_ipv6_literal(b"::0db8:0000:0000:0000:ff00:0042:8329", 0).unwrap(),
			Some((0x0000_0db8_0000_0000_0000_ff00_0042_8329, 36))
		);
	}

	#[test]
	fn ipv6_rule2_2() {
		assert_eq!(
			parse_ipv6_literal(b"::0db8:0000:0000:0000:ff00:0.66.131.41", 0).unwrap(),
			Some((0x0000_0db8_0000_0000_0000_ff00_0042_8329, 38))
		);
	}

	#[test]
	fn ipv6_rule3_1() {
		assert_eq!(
			parse_ipv6_literal(b"2001::0000:0000:0000:ff00:0042:8329", 0).unwrap(),
			Some((0x2001_0000_0000_0000_0000_ff00_0042_8329, 35))
		);
	}

	#[test]
	fn ipv6_rule3_2() {
		assert_eq!(
			parse_ipv6_literal(b"2001::0000:0000:0000:ff00:0042:8329", 0).unwrap(),
			Some((0x2001_0000_0000_0000_0000_ff00_0042_8329, 35))
		);
	}

	#[test]
	fn ipv6_rule4_1() {
		assert_eq!(
			parse_ipv6_literal(b"2001::0000:0000:ff00:0042:8329", 0).unwrap(),
			Some((0x2001_0000_0000_0000_0000_ff00_0042_8329, 30))
		);
	}

	#[test]
	fn ipv6_rule8_1() {
		assert_eq!(
			parse_ipv6_literal(b"2001:0db8:0000:0000:0000:ff00::8329", 0).unwrap(),
			Some((0x2001_0db8_0000_0000_0000_ff00_0000_8329, 35))
		);
	}

	#[test]
	fn ipv6_invalid1() {
		assert_eq!(
			parse_ipv6_literal(b"2001:0db8:0000:0000:0000:ff00:0042:8329:", 0).unwrap(),
			None
		);
	}

	#[test]
	fn ipv6_invalid2() {
		assert_eq!(
			parse_ipv6_literal(b"2001:0db8:0000:0000:0000:ff00:0042:8329:ffff", 0).unwrap(),
			None
		);
	}

	#[test]
	fn ipv6_invalid3() {
		assert_eq!(parse_ipv6_literal(b"1.1.1.1", 0).unwrap(), None);
	}

	#[test]
	fn ipvfuture_1() {
		assert_eq!(
			parse_ipvfuture_literal(b"v1234.abcdef:ghij:klmno:pqrst", 0).unwrap(),
			Some(29)
		);
	}
}
