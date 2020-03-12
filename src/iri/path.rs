use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use pct_str::PctStr;
use crate::{parsing, IriRefBuf};
use super::Error;

pub struct Path<'a> {
	/// The path slice.
	///
	/// Note that contrarily to the [`Authority`] struct,
	/// this only contains the path slice, and NOT the whole IRI.
	pub(crate) data: &'a [u8]
}

impl<'a> Path<'a> {
	/// Get the underlying path slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

	/// Get the underlying path slice as a percent-encoded string slice.
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
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
		self.data.len() > 1 && self.data.last() == Some(&0x2f)
	}

	pub fn is_closed(&self) -> bool {
		!self.is_open()
	}

	/// Checks if the path starts with `//`.
	///
	/// This is used to check if the path part can be confused with the authority part.
	pub fn is_authority_alike(&self) -> bool {
		self.data.len() >= 2 && self.data[0] == 0x2f && self.data[1] == 0x2f
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

	fn next(&mut self) -> Option<&'a PctStr> {
		let mut start = self.offset;
		let mut end = self.offset;

		loop {
			match parsing::get_char(self.data, end).unwrap() {
				Some(('/', 1)) => {
					if end == self.offset {
						start += 1;
						end += 1;
					} else {
						break
					}
				},
				Some((_, len)) => {
					end += len;
				},
				None => break
			}
		}

		self.offset = end;

		if end > start {
			unsafe {
				Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[start..end])))
			}
		} else {
			None
		}
	}
}

impl<'a> fmt::Display for Path<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Path<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Path<'a> {
	fn eq(&self, other: &Path) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Path<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Path<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_pct_str() == *other
	}
}

impl<'a> Hash for Path<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

pub struct PathMut<'a> {
	pub(crate) buffer: &'a mut IriRefBuf
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
	pub fn open(&mut self) {
		if !self.is_empty() && self.is_closed() {
			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
			self.buffer.p.path_len += 1;
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
	pub fn components(&'a self) -> Components<'a> {
		self.buffer.path().components()
	}

	/// Add a component at the end of the path.
	pub fn push<S: AsRef<[u8]> + ?Sized>(&mut self, component: &S) -> Result<(), Error> {
		let component = component.as_ref();

		let component_len = parsing::parse_path_component(component, 0)?;
		if component_len != component.len() {
			return Err(Error::Invalid);
		}

		if component.is_empty() {
			if self.buffer.path().as_str() == "/" {
				// This is the edge case!
				// We can't have the path starting with `//` without an explicit authority part.
				// So we make sure the authority fragment is showing with `://`.
				self.buffer.authority_mut().make_explicit();
			} else {
				// make sure it ends with a slash.
				self.open();
			}

			// add a slash at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, &[0x2f]);
			self.buffer.p.path_len += 1;
		} else {
			self.open();
			// add the component at the end.
			let offset = self.buffer.p.path_offset() + self.buffer.p.path_len;
			self.buffer.replace(offset..offset, component);
			self.buffer.p.path_len += component.len();
		}

		Ok(())
	}

	pub fn pop(&mut self) -> Result<(), Error> {
		if !self.is_empty() {
			let end = self.buffer.p.path_offset() + self.buffer.p.path_len;
			let mut start = end - 1;

			// We remove the terminating `/`.
			if self.is_open() {
				start -= 1;
			}

			// Find the last component start position.
			while self.buffer.data[start] != 0x2f {
				start -= 1;
			}

			// Do not remove the root `/`.
			if start == self.buffer.p.path_offset() {
				start += 1;
			}

			self.buffer.replace(start..end, &[]);
			self.buffer.p.path_len -= end - start;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::{Iri, IriBuf};

	#[test]
	fn empty() {
		let iri = Iri::new("scheme:").unwrap();
		let path = iri.path();

		assert!(path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.is_closed());
		assert!(path.components().next().is_none());
	}

	#[test]
	fn empty_absolute() {
		let iri = Iri::new("scheme:/").unwrap();
		let path = iri.path();

		assert!(path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_closed());
		assert!(path.components().next().is_none());
	}

	#[test]
	fn non_empty() {
		let iri = Iri::new("scheme:a/b").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.is_closed());

		let mut components = path.components();
		assert!(components.next().unwrap() == "a");
		assert!(components.next().unwrap() == "b");
		assert!(components.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let iri = Iri::new("scheme:/foo/bar").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_closed());

		let mut components = path.components();
		assert!(components.next().unwrap() == "foo");
		assert!(components.next().unwrap() == "bar");
		assert!(components.next().is_none());
	}

	#[test]
	fn is_open() {
		let iri = Iri::new("scheme:/red/green/blue/").unwrap();
		let path = iri.path();

		assert!(!path.is_empty());
		assert!(path.is_absolute());
		assert!(path.is_open());

		let mut components = path.components();
		assert!(components.next().unwrap() == "red");
		assert!(components.next().unwrap() == "green");
		assert!(components.next().unwrap() == "blue");
		assert!(components.next().is_none());
	}

	#[test]
	fn push() {
		let mut iri = IriBuf::new("scheme:foo").unwrap();
		let mut path = iri.path_mut();

		path.push("bar").unwrap();

		assert_eq!(iri.as_str(), "scheme:foo/bar");
	}

	#[test]
	fn push_empty_component() {
		let mut iri = IriBuf::new("scheme:foo/bar").unwrap();
		let mut path = iri.path_mut();

		path.push("").unwrap();

		assert_eq!(iri.as_str(), "scheme:foo/bar//");
	}

	#[test]
	fn push_empty_component_edge_case() {
		let mut iri = IriBuf::new("scheme:/").unwrap();
		let mut path = iri.path_mut();

		path.push("").unwrap();

		assert_eq!(iri.as_str(), "scheme:////");
	}

	#[test]
	fn pop() {
		let mut iri = IriBuf::new("scheme:foo/bar").unwrap();
		let mut path = iri.path_mut();

		path.pop().unwrap();

		assert_eq!(iri.as_str(), "scheme:foo");
	}

	#[test]
	fn pop_open() {
		let mut iri = IriBuf::new("scheme:foo/bar/").unwrap();
		let mut path = iri.path_mut();

		path.pop().unwrap();

		assert_eq!(iri.as_str(), "scheme:foo");
	}

	#[test]
	fn pop_open_empty_component() {
		let mut iri = IriBuf::new("scheme:foo//").unwrap();
		let mut path = iri.path_mut();

		path.pop().unwrap();

		assert_eq!(iri.as_str(), "scheme:foo");
	}

	#[test]
	fn pop_open_empty_component_edge_case() {
		let mut iri = IriBuf::new("scheme:////").unwrap();
		let mut path = iri.path_mut();

		path.pop().unwrap();

		assert_eq!(iri.as_str(), "scheme:///");
	}

	#[test]
	fn open() {
		let mut iri = IriBuf::new("scheme:/a").unwrap();
		let mut path = iri.path_mut();

		path.open();

		assert_eq!(iri.as_str(), "scheme:/a/");
	}
}
