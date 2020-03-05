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
	userinfo_len: usize,
	host_offset: usize, // relative to offset
	host_len: usize,
	port_len: usize
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

pub fn parse_host(buffer: &[u8], i: usize) -> Result<usize, Error> {
	// TODO
}

pub fn parse_port(buffer: &[u8], i: usize) -> Result<usize, Error> {
	// TODO
}

/// Parse the IRI authority.
pub fn parse_authority(buffer: &[u8], mut i: usize) -> Result<(usize, usize, usize, usize), Error> {
	let offset = i;
	let mut userinfo_len = 0;
	let mut host_offset = i;
	let mut host_len;
	let mut port_len = 0;

	if let Some(len) = parse_userinfo(buffer, i)? {
		userinfo_len = len;
		host_offset = len + 1;
		i += host_offset
	}

	host_len = parse_host(buffer, i)?;
	port_len = match get_char(buffer, i + host_len)? {
		Some((':', 1)) => parse_port(buffer, i + host_len + 1)?,
		_ => 0
	}

	Ok(Authority {
		offset, userinfo_len, host_offset, host_len, port_len
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
