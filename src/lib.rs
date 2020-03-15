pub mod parsing;
mod iri;
mod reference;

use std::ops::Range;
pub use crate::iri::*;
pub use crate::reference::*;

pub(crate) fn replace(buffer: &mut Vec<u8>, range: Range<usize>, content: &[u8]) {
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
		} else { // grow
			let tail_len = buffer.len() - range.end;

			buffer.resize(new_end + tail_len, 0);

			for i in 0..tail_len {
				buffer[new_end + tail_len - i - 1] = buffer[range.end + tail_len - i - 1];
			}
		}
	}

	// actually replace the content.
	for i in 0..content.len() {
		buffer[range.start + i] = content[i]
	}
}
