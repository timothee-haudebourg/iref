pub fn get_byte(buffer: &[u8], i: usize) -> Option<u32> {
	match buffer.get(i) {
		Some(c) => Some(*c as u32),
		None => None,
	}
}

pub fn expect_byte(buffer: &[u8], i: usize) -> Result<u32, ()> {
	get_byte(buffer, i).ok_or(())
}

/// Return a char and the size of its UTF-8 encoding.
pub fn get_codepoint(buffer: &[u8], i: usize) -> Result<Option<(u32, u8)>, ()> {
	if let Some(a) = get_byte(buffer, i) {
		let r = if a & 0x80 == 0x00 {
			(a, 1)
		} else if a & 0xE0 == 0xC0 {
			let b = expect_byte(buffer, i + 1)?;
			((a & 0x1F) << 6 | b, 2)
		} else if a & 0xF0 == 0xE0 {
			let b = expect_byte(buffer, i + 1)?;
			let c = expect_byte(buffer, i + 2)?;
			((a & 0x0F) << 12 | b << 6 | c, 2)
		} else if a & 0xF8 == 0xF0 {
			let b = expect_byte(buffer, i + 1)?;
			let c = expect_byte(buffer, i + 2)?;
			let d = expect_byte(buffer, i + 3)?;
			((a & 0x07) << 18 | b << 12 | c << 6 | d, 3)
		} else {
			return Err(());
		};

		Ok(Some(r))
	} else {
		return Ok(None);
	}
}

pub fn get_char(buffer: &[u8], i: usize) -> Result<Option<(char, u8)>, ()> {
	match get_codepoint(buffer, i) {
		Ok(Some((codepoint, len))) => match std::char::from_u32(codepoint) {
			Some(c) => Ok(Some((c, len))),
			None => Err(()),
		},
		Ok(None) => Ok(None),
		Err(()) => Err(()),
	}
}
