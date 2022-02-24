pub fn get_byte(buffer: &[u8], i: usize) -> Option<u32> {
	buffer.get(i).map(|c| *c as u32)
}

pub fn expect_byte(buffer: &[u8], i: usize) -> Result<u32, ()> {
	get_byte(buffer, i).ok_or(())
}

/// Return a char and the size of its UTF-8 encoding.
pub fn get_codepoint(buffer: &[u8], i: usize) -> Result<Option<(u32, u8)>, ()> {
	if let Some(a) = get_byte(buffer, i) {
		let r = if a & 0x80 == 0x00 {
			(a, 1)
		} else if a & 0xe0 == 0xc0 {
			let b = expect_byte(buffer, i + 1)?;
			((a & 0x1f) << 6 | (b & 0x3f), 2)
		} else if a & 0xf0 == 0xe0 {
			let b = expect_byte(buffer, i + 1)?;
			let c = expect_byte(buffer, i + 2)?;
			((a & 0x0f) << 12 | (b & 0x3f) << 6 | (c & 0x3f), 3)
		} else if a & 0xf8 == 0xf0 {
			let b = expect_byte(buffer, i + 1)?;
			let c = expect_byte(buffer, i + 2)?;
			let d = expect_byte(buffer, i + 3)?;
			(
				(a & 0x07) << 18 | (b & 0x3f) << 12 | (c & 0x3f) << 6 | (d & 0x3f),
				4,
			)
		} else {
			return Err(());
		};

		Ok(Some(r))
	} else {
		Ok(None)
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

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	fn decode() {
		let string = "伝言/أكرم";
		let bytes = string.as_bytes();

		let mut index = 0;
		let mut decoded = String::new();
		while index < bytes.len() {
			let (c, i) = get_char(bytes, index).unwrap().unwrap();
			decoded.push(c);
			index += i as usize;
		}

		assert_eq!(decoded, string)
	}
}
