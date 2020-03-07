use crate::utf8;

#[derive(Debug)]
enum Error {
	/// The input data is not a valid UTF-8 encoded string.
	InvalidEncoding,

	Invalid,

	InvalidData
}

struct Authority {
	offset: usize,
	userinfo_len: Option<usize>,
	host_len: Option<usize>,
	port_len: Option<usize>
}

struct Iri<'a> {
	data: &'a [u8],
	scheme_len: usize,
	authority: Authority,
	path_len: usize
}

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]>>(buffer: &'a [u8]) -> Result<Iri<'a>> {
		let scheme_len = parse_scheme(buffer, 0)?;
		let mut authority_offset = 0;
		let mut authority_len = 0;
		let mut path_len = 0;
		expect(buffer, ':')?;

		match get_char(buffer, scheme_len + 1)? {
			Some(('/', 1)) => {
				match get_char(buffer, scheme_len + 2)? {
					Some(('/', 1)) => {
						authority_offset = scheme_len + 3;
						let authority_end = parse_authority(buffer, authority_offset)?;
						let path_end = parse_path(buffer, authority_end, true, false)?;

						authority_len = authority_end - authority_offset;
						path_len = path_end - authority_end;
					},
					_ => {
						authority_offset = scheme_len + 2;
						let path_end = parse_path(buffer, authority_offset, true, true)?;
						path_len = authority_end - authority_offset;
					}
				}
			},
			_ => {
				authority_offset = scheme_len + 1;
				let path_end = parse_path(buffer, authority_offset, false, true)?;
				path_len = authority_end - authority_offset;
			}
		}

		Ok(Iri {
			data: buffer,
			scheme_len, authority_offset, authority_len, path_len
		})
	}

	pub fn scheme(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(data[0..self.scheme_len])
		}
	}

	pub fn authority(&self) -> Option<&str> {
		if self.authority_len > 0 {
			unsafe {
				Ok(std::str::from_utf8_unchecked(data[self.authority_offset..(self.authority_offset+self.scheme_len)]))
			}
		} else {
			None
		}
	}

	pub fn path(&self) -> Option<&str> {
		if self.path_len > 0 {
			unsafe {
				let offset = self.authority_offset + self.authority_len;
				Ok(std::str::from_utf8_unchecked(data[offset..(offset+self.path_len)]))
			}
		} else {
			None
		}
	}
}

pub fn get_char(buffer: &[u8], i: usize) -> Result<usize, Error> {
	match utf8::get_char(buffer, i) {
		Ok(r) => r,
		Err(_) => Err(Error::InvalidEncoding)
	}
}

pub fn expect(buffer: &[u8], c: char, i: usize) -> Result<(), Error> {
	match get_char(buffer, i)? {
		Some((found_c, 1)) => {
			if found_c == c {
				Ok(())
			} else {
				Err(Error::Invalid)
			}
		},
		None => Err(Error::Invalid)
	}
}

/// Parse the IRI scheme.
pub fn parse_scheme(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	loop {
		match get_char(buffer, i)? {
			Some((c, len)) if (i == 0 && c.is_alphabetic()) || (i > 0 && (c.is_alphanumeric() || c == '+' || c == '-' || c == '.' || c == ':')) => {
				i += len
			},
			_ => break
		}
	}

	Ok(i)
}

pub fn is_ucschar(c: char) -> bool {
	let c = c as u32;
	c >= 0xA0 && c <= 0xD7FF || c >= 0xF900 && c <= 0xFDCF || c >= 0xFDF0 && c <= 0xFFEF ||
	c >= 0x10000 && c <= 0x1FFFD ||
	c >= 0x20000 && c <= 0x2FFFD ||
	c >= 0x30000 && c <= 0x3FFFD ||
	c >= 0x40000 && c <= 0x4FFFD ||
	c >= 0x50000 && c <= 0x5FFFD ||
	c >= 0x60000 && c <= 0x6FFFD ||
	c >= 0x70000 && c <= 0x7FFFD ||
	c >= 0x80000 && c <= 0x8FFFD ||
	c >= 0x90000 && c <= 0x9FFFD ||
	c >= 0xA0000 && c <= 0xAFFFD ||
	c >= 0xB0000 && c <= 0xBFFFD ||
	c >= 0xC0000 && c <= 0xCFFFD ||
	c >= 0xD0000 && c <= 0xDFFFD ||
	c >= 0xE1000 && c <= 0xEFFFD
}

pub fn is_unreserved(c: char) -> bool {
	c.is_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~' || is_ucschar(c)
}

pub fn is_subdelim(c: char) -> bool {
	match c {
		'!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' => true,
		_ => false
	}
}

pub fn is_hex_digit(buffer: &[u8], mut i: usize) -> bool {
	match get_char(buffer, i)? {
		Some((c, 1)) => {
			c.to_digit(16).is_some()
		},
		None => false
	}
}

pub fn parse_pct_encoded(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	match get_char(buffer, i)? {
		Some(('%', 1)) => {
			if is_hex_digit(buffer, i+1) && is_hex_digit(buffer, i+2) {
				Ok(Some(3))
			} else {
				Err(Error::InvalidPCTEncoded)
			}
		},
		None => Ok(None)
	}
}

pub fn parse_userinfo(buffer, &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	loop {
		match get_char(buffer, i)? {
			Some(('@', 1)) => {
				return Ok(Some(i))
			},
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i) {
					i += len
				} else {
					break
				}
			},
			Some((c, len)) if c == ':' || is_subdelim(c) || is_unreserved(c) => {
				i += len
			},
			_ => break
		}
	}

	Ok(None)
}

pub fn parse_dec_octet(buffer: &[u8], i: usize) -> Result<Option<(u32, usize)>, Error> {
	let octet = 0u32;
	let len = 0;

	loop {
		match get_char(buffer, i + len)? {
			Some((c, 1)) => {
				if let Some(d) = c.to_digit(10) {
					if octet == 25 && d > 5 {
						return Ok(None);
					} else {
						octet = octet * 10 + d
					}

					len += 1;

					if len >= 3 || octet > 25 {
						break
					}
				} else {
					break
				}
			},
			_ => break
		}
	}

	if len == 0 {
		Ok(None)
	} else {
		Ok(Some((octet, len)))
	}
}

/// Parse an IPv4 literal.
pub fn parse_ipv4_literal(buffer: &[u8], mut i: usize) -> Result<Option<(u32, usize)>> {
	let offset = i;
	if let Some((a, olen)) = parse_dec_octet(buffer, i)? {
		i += olen;
		if let Some(('.', 1)) = get_char(buffer, i)? {
			i += 1;
			if let Some((a, olen)) = parse_dec_octet(buffer, i)? {
				i += olen;
				if let Some(('.', 1)) = get_char(buffer, i)? {
					i += 1;
					if let Some((a, olen)) = parse_dec_octet(buffer, i)? {
						i += olen;
						if let Some(('.', 1)) = get_char(buffer, i)? {
							i += 1;
							if let Some((a, olen)) = parse_dec_octet(buffer, i)? {
								i += olen;
								let ipv4 = (a << 24) | (b << 16) | (c << 8) | d
								let len = i - offset;
								return Ok(Some((ipv4, len)))
							}
						}
					}
				}
			}
		}
	}

	Ok(None)
}

pub fn parse_h16(buffer: &[u8], mut i: usize) -> Result<Option<(u16, usize)>, Error> {
	let len = 0;
	let h16 = 0;

	loop {
		match get_char(buffer, i + len)? {
			Some((c, 1)) => {
				if let Some(d) = c.to_digit(16) {
					h16 = (h16 << 4) | d;
					len += 1;
					if len >= 4 {
						break
					}
				} else {
					break
				}
			},
			_ => break
		}
	}

	Ok(Some((h16, len)))
}

/// Parse an IPv6 literal.
/// Return the IPv6 and the string length.
pub fn parse_ipv6_literal(buffer: &[u8], mut i: usize) -> Result<Option<(u128, usize)>, Error> {
	let mut lhs = 0u128;
	let mut lhs_count = 0;

	let mut lit = 0u128;
	let mut lit_count = 0;

	let is_lhs = true;
	let offset = i;

	loop {
		if lhs_count + lit_count >= 8 {
			return Ok(None);
		}

		if is_lhs {
			if let Some((':', 1)) = get_char(buffer, i) {
				i += 1;

				if lhs_count == 0 {
					if let Some((':', 1)) = get_char(buffer, i) {
						i += 1;
					} else {
						return Ok(None); // Invalid IPv6 (unexpected char)
					}
				}

				lhs = lit;
				lhs_count += lit_count;

				is_lhs = false;

				lit = 0;
				lit_count = 1;
				continue;
			}
		}

		if lhs_count + lit_count <= 6 {
			if let Some((ipv4, len)) = parse_ipv4_literal(buffer, i) {
				lit = (lit << 32) | n;
				lit_count += 2;
				i += len;
				break
			}
		}

		if let Some((n, len)) = parse_h16(buffer, i) {
			lit = (lit << 16) | n;
			lit_count += 1;
			i += len;

			match get_char(buffer, i) {
				Some((']', 1)) => {
					break
				},
				Some((':', 1)) => {
					i += 1
				},
				_ => {
					return Ok(None); // Invalid IPv6 (unexpected char)
				}
			}
		} else {
			return Ok(None); // Invalid IPv6 (unexpected char)
		}
	}

	i += 1;
	lit = lit | (lhs << (16 * (8 - lhs_count)));
	let len = i - offset;
	Ok(Some((lit, len)))
}

pub fn parse_ip_literal(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	let offset = i;
	if let Some(('[', 1)) = get_char(buffer, i)? {
		i += 1;
		if let Some((_, l)) = parse_ipv6_literal(buffer, i)? {
			i += l;
		} else {
			return Ok(None) // TODO Ipv future
		}

		if let Some((']', 1)) = get_char(buffer, i)? {
			i += 1;
			let len = i - offset;
			return Ok(Some(len))
		}
	}

	Ok(None)
}

pub fn parse_ireg_name(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	let offset = i;
	loop {
		match get_char(buffer, i)? {
			Some(('%', 1)) => {
				if let Some(len) = parse_pct_encoded(buffer, i)? {
					i += len
				} else {
					break
				}
			},
			Some((c, len)) if is_subdelim(c) || is_unreserved(c) => {
				i += len
			},
			_ => break
		}
	}

	let len = i - offset;

	if len > 0 {
		Ok(Some(len))
	} else {
		Ok(None)
	}
}

pub fn parse_host(buffer: &[u8], i: usize) -> Result<usize, Error> {
	if let Some(len) = parse_ip_literal(buffer, i) {
		Ok(len)
	} else if let Some(len) = parse_ipv4_literal(buffer, i) {
		Ok(len)
	} else {
		parse_ireg_name(buffer, i)
	}
}

pub fn parse_port(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let offset = i;
	loop {
		match get_char(buffer, i)? {
			Some((c, 1)) => {
				if let Some(d) = c.to_digit(10) {
					i += len
				} else {
					break
				}
			},
			_ => break
		}
	}

	let len = i - offset;

	if len > 0 {
		Ok(Some(len))
	} else {
		Ok(None)
	}
}

/// Parse the IRI authority.
pub fn parse_authority(buffer: &[u8], mut i: usize) -> Result<Authority, Error> {
	let offset = i;
	let mut userinfo_len = None;
	let mut host_len = None;
	let mut port_len;

	if let Some(len) = parse_userinfo(buffer, i)? {
		userinfo_len = Some(len);
		i += len + 1;
	}

	if let Some(len) = parse_host(buffer, i)? {
		host_len = Some(len);
		i += len;
	}

	port_len = match get_char(buffer, i)? {
		Some((':', 1)) => {
			i += 1;
			if let Some(len) = parse_port(buffer, i)? {
				i += len;
				Some(len)
			} else {
				Some(0)
			}
		},
		_ => None
	}

	Ok(Authority {
		offset, userinfo_len, host_len, port_len
	})
}

/// Parse IRI path.
pub fn parse_path(buffer: &[u8], mut i: usize, mut absolute: bool, non_empty: bool) -> Result<usize, Error> {
	let start = i;

	match get_char(buffer, i)? {
		None | Some(('?', _)) | Some(('#', _)) => break,
		Some(('/', len)) if i == start => {
			if absolute {
				return Err(Error::InvalidPath);
			} else {
				absolute = true;
			}

			i += len
		},
		Some((_, len)) => {
			i += len
		}
	}

	Ok(i)
}
