pub struct Path<'a> {
	data: &'a [u8]
}

impl<'a> Path<'a> {
	/// Get the underlying path slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty() || self.data == &[0x2f]
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	pub fn is_absolute(&self) -> bool {
		!self.data.is_empty() && self.data[0] == 0x2f
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	pub fn is_relative(&self) -> bool {
		self.data.is_empty() || self.data[0] != 0x2f
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	pub fn is_open(&self) -> bool {
	 	self.data.len() > 1 && self.data.last() == 0x2f
	}

	pub fn is_closed(&self) -> bool {
		!self.data.is_open()
	}

	/// Produces an iterator over the components of the IRI path.
	///
	/// Note that this is an IRI path, not an IRI reference path: no normalization occurs with
	/// `.` and `..` components. This is done by the IRI reference resolution function.
	///
	/// Empty components are preserved: the path `a//b` will raise the three components `a`, `` and
	/// `b`.
	/// The absolute path `/` has no components, but the path `/a/` has two components, `a` and ``.
	pub fn components(&self) -> Components<'a> {
		Components {
			data: &self.data,
			offset: 0
		}
	}
}

pub struct Components<'a> {
	data: &'a [u8],
	offset: usize
}

impl<'a> Iterator for Components<'a> {
	type Item = &'a PctStr;

	fn next(&self) -> &'a PctStr {
		let start = self.offset;

		loop {
			match get_char(self.data, i) {
				Some(('/', 1)) => {
					if start == self.offset {
						self.offset += 1;
					} else {
						break
					}
				},
				Some((_, len)) => {
					self.offset += len;
				},
				None => break
			}
		}

		if start > self.offset {
			unsafe {
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[start..self.offset])))
			}
		} else {
			None
		}
	}
}

pub struct PathMut<'a> {
	buffer: &'a mut IriBuf
}

impl<'a> PathMut<'a> {
	/// Checks if the path is empty.
	///
	/// Returns `true` if the path is `` or `/`.
	pub fn is_empty(&self) -> bool {
		self.buffer.path().is_empty()
	}

	/// Checks if the path is absolute.
	///
	/// A path is absolute if it starts with a `/`.
	/// A path is necessarily absolute if the IRI it is contained in contains a non-empty
	/// authority.
	pub fn is_absolute(&self) -> bool {
		self.buffer.path().is_absolute()
	}

	/// Checks if the path is relative.
	///
	/// A path is relative if it does not start with a `/`.
	/// A path cannot be relative if the IRI it is contained in contains a non-empty authority.
	pub fn is_relative(&self) -> bool {
		self.buffer.path().is_relative()
	}

	/// Checks if the path ends with a `/` but is not equal to `/`.
	pub fn is_open(&self) -> bool {
	 	self.buffer.path().is_open()
	}

	pub fn is_closed(&self) -> bool {
		self.buffer.path().is_closed()
	}

	/// Make sure the last component is followed by a `/`.
	///
	/// This has no effect if the path is empty.
	pub fn open(&self) {
		if !self.is_empty() && self.is_closed() {
			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
		}
	}

	/// Produces an iterator over the components of the IRI path.
	///
	/// Note that this is an IRI path, not an IRI reference path: no normalization occurs with
	/// `.` and `..` components. This is done by the IRI reference resolution function.
	///
	/// Empty components are preserved: the path `a//b` will raise the three components `a`, `` and
	/// `b`.
	/// The absolute path `/` has no components, but the path `/a/` has two components, `a` and ``.
	pub fn components(&self) -> Components<'a> {
		self.buffer.path().components()
	}

	/// Add a component at the end of the path.
	pub fn push<S: AsRef<[u8]> + ?Sized>(&mut self, component: &S) -> Result<(), Error> {
		let component = component.as_ref();

		/// TODO parse the component.

		if component.is_empty() {
			if self.path().as_str() == "/" {
				// This is the edge case!
				// We can't have the path starting with `//` without an explicit authority part.
				// So we make sure the authority fragment is showing with `://`.
				self.buffer.make_authority_explicit();
			} else {
				// make sure it ends with a slash.
				self.open();
			}

			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
		} else {
			self.open();
			// add the component at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, component);
		}
	}

	pub fn pop<S: AsRef<[u8]> + ?Sized>(&mut self) -> Result<(), Error> {
		// TODO
	}
}
