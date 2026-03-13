use std::ops::Range;

pub fn allocate_range(buffer: &mut Vec<u8>, range: Range<usize>, len: usize) {
	let range_len = range.end - range.start;

	// move the content around.
	if range_len != len {
		let new_end = range.start + len;

		if range_len > len {
			// shrink
			buffer.copy_within(range.end.., new_end);
			buffer.truncate(new_end + (buffer.len() - range.end));
		} else {
			// grow
			let old_len = buffer.len();
			buffer.resize(old_len + (len - range_len), 0);
			buffer.copy_within(range.end..old_len, new_end);
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
	buffer[start..(start + content.len())].copy_from_slice(content)
}
