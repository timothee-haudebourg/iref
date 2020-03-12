mod utf8;

use log::*;

#[derive(Debug)]
pub enum Error {
	/// The input data is not a valid UTF-8 encoded string.
	InvalidEncoding,

	Invalid,

	InvalidPath,

	InvalidPCTEncoded
}

#[derive(Clone, Copy)]
pub struct ParsedAuthority {
	pub offset: usize,
	pub userinfo_len: Option<usize>,
	pub host_len: usize,
	pub port_len: Option<usize>
}

impl ParsedAuthority {
	pub fn is_empty(&self) -> bool {
		self.userinfo_len.is_none() && self.host_len == 0 && self.port_len.is_none()
	}

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

	pub fn host_offset(&self) -> usize {
		let mut offset = self.offset;

		if let Some(l) = self.userinfo_len {
			offset += l + 1;
		}

		offset
	}

	pub fn port_offset(&self) -> usize {
		let mut offset = self.offset;

		if let Some(l) = self.userinfo_len {
			offset += l + 1;
		}

		offset += self.host_len;

		if let Some(_) = self.port_len {
			offset += 1;
		}

		offset
	}
}

#[derive(Clone, Copy)]
pub struct ParsedIri {
	pub scheme_len: usize,
	pub authority: ParsedAuthority,
	pub path_len: usize,
	pub query_len: Option<usize>,
	pub fragment_len: Option<usize>
}

impl ParsedIri {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<ParsedIri, Error> {
		let buffer = buffer.as_ref();
		let scheme_len = parse_scheme(buffer, 0)?;
		let mut authority = ParsedAuthority {
			offset: 0,
			userinfo_len: None,
			host_len: 0,
			port_len: None
		};
		let path_len;
		let mut query_len = None;
		let mut fragment_len = None;
		expect(buffer, ':', scheme_len)?;

		match get_char(buffer, scheme_len + 1)? {
			Some(('/', 1)) => {
				match get_char(buffer, scheme_len + 2)? {
					Some(('/', 1)) => {
						authority.offset = scheme_len + 3;
						authority = parse_authority(buffer, authority.offset)?;
						let authority_end = authority.offset + authority.len();
						// path must be absolute.
						path_len = if let Some(('/', 1)) = get_char(buffer, authority_end)? {
							parse_path(buffer, authority_end)?
						} else {
							0
						};
					},
					_ => {
						authority.offset = scheme_len + 1;
						path_len = parse_path(buffer, authority.offset)?;
					}
				}
			},
			_ => {
				authority.offset = scheme_len + 1;
				path_len = parse_path(buffer, authority.offset)?;
			}
		}

		let i = authority.offset + authority.len() +  path_len;

		match get_char(buffer, i)? {
			Some(('#', 1)) => {
				fragment_len = Some(parse_fragment(buffer, i+1)?)
			},
			Some(('?', 1)) => {
				let len = parse_query(buffer, i+1)?;
				query_len = Some(len);
				match get_char(buffer, i + 1 + len)? {
					Some(('#', 1)) => {
						fragment_len = Some(parse_fragment(buffer, i + 1 + len + 1)?)
					},
					Some(_) => return Err(Error::InvalidPath),
					None => (),
				}
			},
			Some(_) => return Err(Error::InvalidPath),
			None => (),
		}

		Ok(ParsedIri {
			scheme_len, authority, path_len, query_len, fragment_len
		})
	}

	pub fn len(&self) -> usize {
		let mut offset = self.authority.offset + self.authority.len() + self.path_len;

		if let Some(len) = self.query_len {
			offset += 1 + len;
		}

		if let Some(len) = self.fragment_len {
			offset += 1 + len;
		}

		offset
	}

	pub fn path_offset(&self) -> usize {
		self.authority.offset + self.authority.len()
	}

	pub fn query_offset(&self) -> usize {
		let mut offset = self.authority.offset + self.authority.len() + self.path_len;

		if let Some(_) = self.query_len {
			offset += 1;
		}

		offset
	}

	pub fn fragment_offset(&self) -> usize {
		let mut offset = self.authority.offset + self.authority.len() + self.path_len;

		if let Some(len) = self.query_len {
			offset += 1 + len;
		}

		if let Some(_) = self.fragment_len {
			offset += 1;
		}

		offset
	}
}

pub fn get_char(buffer: &[u8], i: usize) -> Result<Option<(char, usize)>, Error> {
	match utf8::get_char(buffer, i) {
		Ok(None) => Ok(None),
		Ok(Some((c, len))) => Ok(Some((c, len as usize))),
		Err(_) => Err(Error::InvalidEncoding)
	}
}

fn expect(buffer: &[u8], c: char, i: usize) -> Result<(), Error> {
	match get_char(buffer, i)? {
		Some((found_c, 1)) => {
			if found_c == c {
				Ok(())
			} else {
				Err(Error::Invalid)
			}
		},
		_ => Err(Error::Invalid)
	}
}

/// Parse the IRI scheme.
pub fn parse_scheme(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	loop {
		match get_char(buffer, i)? {
			Some((c, len)) if (i == 0 && c.is_alphabetic()) || (i > 0 && (c.is_alphanumeric() || c == '+' || c == '-' || c == '.')) => {
				i += len
			},
			_ => break
		}
	}

	Ok(i)
}

fn is_ucschar(c: char) -> bool {
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

fn is_private(c: char) -> bool {
	let c = c as u32;
	c >= 0xE000 && c <= 0xF8FF ||
	c >= 0xF0000 && c <= 0xFFFFD ||
	c >= 0x100000 && c <= 0x10FFFD
}

fn is_unreserved(c: char) -> bool {
	c.is_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~' || is_ucschar(c)
}

fn is_subdelim(c: char) -> bool {
	match c {
		'!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' => true,
		_ => false
	}
}

fn is_hex_digit(buffer: &[u8], i: usize) -> Result<bool, Error> {
	match get_char(buffer, i)? {
		Some((c, 1)) => {
			Ok(c.to_digit(16).is_some())
		},
		_ => Ok(false)
	}
}

fn parse_pct_encoded(buffer: &[u8], i: usize) -> Result<Option<usize>, Error> {
	match get_char(buffer, i)? {
		Some(('%', 1)) => {
			if is_hex_digit(buffer, i+1)? && is_hex_digit(buffer, i+2)? {
				Ok(Some(3))
			} else {
				Err(Error::InvalidPCTEncoded)
			}
		},
		_ => Ok(None)
	}
}

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
					break
				}
			},
			Some((c, len)) if c == ':' || is_subdelim(c) || is_unreserved(c) => {
				i += len
			},
			_ => break
		}
	}

	Ok(i - offset)
}

pub fn parse_query(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
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
			Some((c, len)) if c == ':' || c == '@' || c == '/' || c == '?' || is_subdelim(c) || is_unreserved(c) || is_private(c) => {
				i += len
			},
			_ => break
		}
	}

	Ok(i - offset)
}

pub fn parse_fragment(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
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
			Some((c, len)) if c == ':' || c == '@' || c == '/' || c == '?' || is_subdelim(c) || is_unreserved(c) => {
				i += len
			},
			_ => break
		}
	}

	Ok(i - offset)
}

fn parse_dec_octet(buffer: &[u8], i: usize) -> Result<Option<(u32, usize)>, Error> {
	let mut octet = 0u32;
	let mut len = 0;

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
fn parse_ipv4_literal(buffer: &[u8], mut i: usize) -> Result<Option<(u32, usize)>, Error> {
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
								let ipv4 = (a << 24) | (b << 16) | (c << 8) | d;
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

fn parse_h16(buffer: &[u8], i: usize) -> Result<Option<(u16, usize)>, Error> {
	let mut len = 0;
	let mut h16 = 0;

	loop {
		match get_char(buffer, i + len)? {
			Some((c, 1)) => {
				if let Some(d) = c.to_digit(16) {
					h16 = (h16 << 4) | d as u16;
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
fn parse_ipv6_literal(buffer: &[u8], mut i: usize) -> Result<Option<(u128, usize)>, Error> {
	let mut lhs = 0u128;
	let mut lhs_count = 0;

	let mut lit = 0u128;
	let mut lit_count = 0;

	let mut is_lhs = true;
	let offset = i;

	loop {
		if lhs_count + lit_count >= 8 {
			return Ok(None);
		}

		if is_lhs {
			if let Some((':', 1)) = get_char(buffer, i)? {
				i += 1;

				if lhs_count == 0 {
					if let Some((':', 1)) = get_char(buffer, i)? {
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
			if let Some((ipv4, len)) = parse_ipv4_literal(buffer, i)? {
				lit = (lit << 32) | ipv4 as u128;
				// lit_count += 2;
				i += len;
				break
			}
		}

		if let Some((n, len)) = parse_h16(buffer, i)? {
			lit = (lit << 16) | n as u128;
			lit_count += 1;
			i += len;

			match get_char(buffer, i)? {
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

	if lhs_count > 0 {
		lit = lit | (lhs << (16 * (8 - lhs_count)));
	}

	let len = i - offset;
	Ok(Some((lit, len)))
}

fn parse_ip_literal(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
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
			debug!("ip literal");
			return Ok(Some(len))
		}
	}

	Ok(None)
}

fn parse_ireg_name(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
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

	Ok(i - offset)
}

pub fn parse_host(buffer: &[u8], i: usize) -> Result<usize, Error> {
	if let Some(len) = parse_ip_literal(buffer, i)? {
		Ok(len)
	} else if let Some((_, len)) = parse_ipv4_literal(buffer, i)? {
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
				if let Some(_) = c.to_digit(10) {
					i += 1
				} else {
					break
				}
			},
			_ => break
		}
	}

	Ok(i - offset)
}

/// Parse the IRI authority.
pub fn parse_authority(buffer: &[u8], mut i: usize) -> Result<ParsedAuthority, Error> {
	let offset = i;
	let mut userinfo_len = None;
	let host_len;
	let port_len;

	let userinfo_tmp_len = parse_userinfo(buffer, i)?;
	if let Some(('@', 1)) = get_char(buffer, i + userinfo_tmp_len)? {
		userinfo_len = Some(userinfo_tmp_len);
		i += userinfo_tmp_len + 1;
	}

	host_len = parse_host(buffer, i)?;
	i += host_len;

	port_len = match get_char(buffer, i)? {
		Some((':', 1)) => {
			i += 1;
			Some(parse_port(buffer, i)?)
		},
		_ => None
	};

	Ok(ParsedAuthority {
		offset, userinfo_len, host_len, port_len
	})
}

/// Parse IRI path.
pub fn parse_path(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let start = i;

	loop {
		match get_char(buffer, i)? {
			None | Some(('?', _)) | Some(('#', _)) => break,
			Some((_, len)) => {
				i += len
			}
		}
	}

	Ok(i - start)
}

/// Parse IRI path component.
pub fn parse_path_component(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
	let start = i;

	loop {
		match get_char(buffer, i)? {
			None | Some(('?', _)) | Some(('#', _)) | Some(('/', _)) => break,
			Some((_, len)) => {
				i += len
			}
		}
	}

	Ok(i - start)
}
