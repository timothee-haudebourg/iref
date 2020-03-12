pub mod parsing;
mod iri;
mod reference;

use std::ops::Range;
pub use crate::iri::*;
pub use crate::reference::*;

pub(crate) fn replace(buffer: &mut Vec<u8>, authority: &mut parsing::ParsedAuthority, range: Range<usize>, content: &[u8]) {
	let range_len = range.end - range.start;

	// move the content around.
	if range_len != content.len() {
		let tail_len = buffer.len() - range.end; // the length of the content in the buffer after [range].
		let new_end = range.start + content.len();

		if range_len > content.len() { // shrink
			for i in 0..tail_len {
				buffer[new_end + i] = buffer[range.end + i];
			}

			buffer.resize(new_end + tail_len, 0);

			if authority.offset > range.end {
				let delta = range_len - content.len();
				authority.offset -= delta;
			}
		} else { // grow
			let tail_len = buffer.len() - range.end;

			buffer.resize(new_end + tail_len, 0);

			for i in 0..tail_len {
				buffer[new_end + tail_len - i - 1] = buffer[range.end + tail_len - i - 1];
			}

			if authority.offset > range.end {
				let delta = content.len() - range_len;
				authority.offset += delta;
			}
		}
	}

	// actually replace the content.
	for i in 0..content.len() {
		buffer[range.start + i] = content[i]
	}
}
