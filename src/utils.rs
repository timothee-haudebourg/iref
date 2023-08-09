use std::ops::Range;

pub fn allocate_range(buffer: &mut Vec<u8>, range: Range<usize>, len: usize) {
	let range_len = range.end - range.start;

	// move the content around.
	if range_len != len {
		let tail_len = buffer.len() - range.end; // the length of the content in the buffer after [range].
		let new_end = range.start + len;

		if range_len > len {
			// shrink
			for i in 0..tail_len {
				buffer[new_end + i] = buffer[range.end + i];
			}

			buffer.resize(new_end + tail_len, 0);
		} else {
			// grow
			let tail_len = buffer.len() - range.end;

			buffer.resize(new_end + tail_len, 0);

			for i in 0..tail_len {
				buffer[new_end + tail_len - i - 1] = buffer[range.end + tail_len - i - 1];
			}
		}
	}
}

/// Replacement function in IRI-reference buffers.
///
/// Replace the given `range` of the input `buffer` with the given `content`.
/// This function is used in many places to replace parts of an IRI-reference buffer data.
pub fn replace(buffer: &mut Vec<u8>, range: Range<usize>, content: &[u8]) {
	let start = range.start;
	allocate_range(buffer, range, content.len());

	// actually replace the content.
	for i in 0..content.len() {
		buffer[start + i] = content[i]
	}
}
