use crate::utf8;

#[derive(Debug)]
pub enum Error {
	/// The input data is not a valid UTF-8 encoded string.
	InvalidEncoding,

	Invalid,

	InvalidPath,

	InvalidPCTEncoded
}

struct AuthorityData {
	offset: usize,
	userinfo_len: Option<usize>,
	host_len: Option<usize>,
	port_len: Option<usize>
}

impl AuthorityData {
	pub fn is_empty(&self) -> bool {
		self.userinfo_len.is_none() && self.host_len.is_none() && self.port_len.is_none()
	}

	pub fn len(&self) -> usize {
		let mut len = 0;

		if let Some(l) = self.userinfo_len {
			len += l + 1;
		}

		if let Some(l) = self.host_len {
			len += l;
		}

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

	// pub fn port_offset(&self) -> usize {
	// 	let mut offset = self.offset;
	//
	// 	if let Some(l) = self.userinfo_len {
	// 		offset += l + 1;
	// 	}
	//
	// 	if let Some(l) = self.host_len {
	// 		offset += l;
	// 	}
	//
	// 	if let Some(_) = self.port_len {
	// 		offset += 1;
	// 	}
	//
	// 	offset
	// }
}

pub struct Iri<'a> {
	data: &'a [u8],
	scheme_len: usize,
	authority: AuthorityData,
	path_len: usize
}

pub struct Authority<'a> {
	data: &'a [u8],
	authority: &'a AuthorityData
}

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		let buffer = buffer.as_ref();
		let scheme_len = parse_scheme(buffer, 0)?;
		let mut authority = AuthorityData {
			offset: 0,
			userinfo_len: None,
			host_len: None,
			port_len: None
		};
		let path_len;
		expect(buffer, ':', scheme_len)?;

		match get_char(buffer, scheme_len + 1)? {
			Some(('/', 1)) => {
				match get_char(buffer, scheme_len + 2)? {
					Some(('/', 1)) => {
						authority.offset = scheme_len + 3;
						authority = parse_authority(buffer, authority.offset)?;
						let authority_end = authority.offset + authority.len();
						let path_end = parse_path(buffer, authority_end, true, false)?;

						// let authority_len = authority_end - authority.offset;
						path_len = path_end - authority_end;
					},
					_ => {
						authority.offset = scheme_len + 2;
						let path_end = parse_path(buffer, authority.offset, true, true)?;
						path_len = path_end - authority.offset;
					}
				}
			},
			_ => {
				authority.offset = scheme_len + 1;
				let path_end = parse_path(buffer, authority.offset, false, true)?;
				path_len = path_end - authority.offset;
			}
		}

		Ok(Iri {
			data: buffer,
			scheme_len, authority, path_len
		})
	}

	pub fn scheme(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.scheme_len])
		}
	}

	pub fn authority(&'a self) -> Option<Authority<'a>> {
		if self.authority.is_empty() {
			None
		} else {
			Some(Authority {
				data: self.data,
				authority: &self.authority
			})
		}
	}

	pub fn path(&self) -> Option<&str> {
		if self.path_len > 0 {
			unsafe {
				let offset = self.authority.offset + self.authority.len();
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+self.path_len)]))
			}
		} else {
			None
		}
	}
}

impl<'a> Authority<'a> {
	pub fn host(&self) -> Option<&str> {
		if let Some(len) = self.authority.host_len {
			unsafe {
				let offset = self.authority.host_offset();
				Some(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)]))
			}
		} else {
			None
		}
	}
}

fn get_char(buffer: &[u8], i: usize) -> Result<Option<(char, usize)>, Error> {
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
fn parse_scheme(buffer: &[u8], mut i: usize) -> Result<usize, Error> {
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

fn parse_userinfo(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
	loop {
		match get_char(buffer, i)? {
			Some(('@', 1)) => {
				return Ok(Some(i))
			},
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

	Ok(None)
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

	i += 1;
	lit = lit | (lhs << (16 * (8 - lhs_count)));
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
			return Ok(Some(len))
		}
	}

	Ok(None)
}

fn parse_ireg_name(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
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

fn parse_host(buffer: &[u8], i: usize) -> Result<Option<usize>, Error> {
	if let Some(len) = parse_ip_literal(buffer, i)? {
		Ok(Some(len))
	} else if let Some((_, len)) = parse_ipv4_literal(buffer, i)? {
		Ok(Some(len))
	} else {
		parse_ireg_name(buffer, i)
	}
}

fn parse_port(buffer: &[u8], mut i: usize) -> Result<Option<usize>, Error> {
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

	let len = i - offset;

	if len > 0 {
		Ok(Some(len))
	} else {
		Ok(None)
	}
}

/// Parse the IRI authority.
fn parse_authority(buffer: &[u8], mut i: usize) -> Result<AuthorityData, Error> {
	let offset = i;
	let mut userinfo_len = None;
	let mut host_len = None;
	let port_len;

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
				// i += len;
				Some(len)
			} else {
				Some(0)
			}
		},
		_ => None
	};

	Ok(AuthorityData {
		offset, userinfo_len, host_len, port_len
	})
}

/// Parse IRI path.
fn parse_path(buffer: &[u8], mut i: usize, mut absolute: bool, non_empty: bool) -> Result<usize, Error> {
	let start = i;

	loop {
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
	}

	Ok(i)
}
