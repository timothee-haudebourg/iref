mod segment;
pub use segment::*;

mod r#mut;
pub use r#mut::*;

crate::common::path_impl!("URI");

/// Parses a URI [`Path`] at compile time.
#[macro_export]
macro_rules! path {
	($value:literal) => {
		match $crate::uri::Path::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI path"),
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let path = Path::EMPTY;
		assert!(path.is_empty());
		assert!(!path.is_absolute());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn empty_absolute() {
		let path = Path::EMPTY_ABSOLUTE;
		assert!(path.is_empty());
		assert!(path.is_absolute());
		assert!(path.segments().next().is_none());
	}

	#[test]
	fn non_empty() {
		let path = Path::new(b"a/b").unwrap();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "a");
		assert!(segments.next().unwrap().as_str() == "b");
		assert!(segments.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let path = Path::new(b"/foo/bar").unwrap();
		assert!(!path.is_empty());
		assert!(path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_bytes() == b"foo");
		assert!(segments.next().unwrap().as_bytes() == b"bar");
		assert!(segments.next().is_none());
	}

	#[test]
	fn next_segment() {
		let vectors: [(&[u8], usize, Option<(&[u8], usize)>); 6] = [
			(b"foo/bar", 0, Some((b"foo", 4))),
			(b"foo/bar", 4, Some((b"bar", 8))),
			(b"foo/bar", 8, None),
			(b"foo/bar/", 8, Some((b"", 9))),
			(b"foo/bar/", 9, None),
			(b"//foo", 1, Some((b"", 2))),
		];

		for (input, offset, expected) in vectors {
			unsafe {
				assert_eq!(
					Path::new(input).unwrap().next_segment_from(offset),
					expected.map(|(e, i)| (Segment::new(e).unwrap(), i))
				)
			}
		}
	}

	#[test]
	fn previous_segment() {
		let vectors: [(&[u8], usize, Option<(&[u8], usize)>); 7] = [
			(b"/foo/bar", 1, None),
			(b"foo/bar", 0, None),
			(b"foo/bar", 4, Some((b"foo", 0))),
			(b"foo/bar", 8, Some((b"bar", 4))),
			(b"foo/bar/", 8, Some((b"bar", 4))),
			(b"foo/bar/", 9, Some((b"", 8))),
			(b"//a/b", 4, Some((b"a", 2))),
		];

		for (input, offset, expected) in vectors {
			unsafe {
				assert_eq!(
					Path::new(input).unwrap().previous_segment_from(offset),
					expected.map(|(e, i)| (Segment::new(e).unwrap(), i))
				)
			}
		}
	}

	#[test]
	fn first_segment() {
		let vectors: [(&[u8], Option<&[u8]>); 4] = [
			(b"", None),
			(b"/", None),
			(b"//", Some(b"")),
			(b"/foo/bar", Some(b"foo")),
		];

		for (input, expected) in vectors {
			assert_eq!(
				Path::new(input).unwrap().first(),
				expected.map(|e| Segment::new(e).unwrap())
			)
		}
	}

	#[test]
	fn segments() {
		let vectors: [(&[u8], &[&[u8]]); 8] = [
			(b"", &[]),
			(b"foo", &[b"foo"]),
			(b"/foo", &[b"foo"]),
			(b"foo/", &[b"foo", b""]),
			(b"/foo/", &[b"foo", b""]),
			(b"a/b/c/d", &[b"a", b"b", b"c", b"d"]),
			(b"a/b//c/d", &[b"a", b"b", b"", b"c", b"d"]),
			(
				b"//a/b/foo//bar/",
				&[b"", b"a", b"b", b"foo", b"", b"bar", b""],
			),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected)
					.all(|(a, b)| a.as_bytes() == *b)
			)
		}
	}

	#[test]
	fn segments_rev() {
		let vectors: [(&[u8], &[&[u8]]); 8] = [
			(b"", &[]),
			(b"foo", &[b"foo"]),
			(b"/foo", &[b"foo"]),
			(b"foo/", &[b"foo", b""]),
			(b"/foo/", &[b"foo", b""]),
			(b"a/b/c/d", &[b"a", b"b", b"c", b"d"]),
			(b"a/b//c/d", &[b"a", b"b", b"", b"c", b"d"]),
			(
				b"//a/b/foo//bar/",
				&[b"", b"a", b"b", b"foo", b"", b"bar", b""],
			),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().rev().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected.into_iter().rev())
					.all(|(a, b)| a.as_bytes() == *b)
			)
		}
	}

	#[test]
	fn normalized() {
		let vectors: [(&[u8], &[u8]); 9] = [
			(b"", b""),
			(b"a/b/c", b"a/b/c"),
			(b"a/..", b""),
			(b"a/b/..", b"a/"),
			(b"a/b/../", b"a/"),
			(b"a/b/c/..", b"a/b/"),
			(b"a/b/c/.", b"a/b/c/"),
			(b"a/../..", b"../"),
			(b"/a/../..", b"/"),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let output = path.normalized();
			assert_eq!(output.as_bytes(), expected);
		}
	}

	#[test]
	fn eq() {
		let vectors: [(&[u8], &[u8]); 11] = [
			(b"a/b/c", b"a/b/c"),
			(b"a/b/c", b"a/b/c/."),
			(b"a/b/c/", b"a/b/c/./"),
			(b"a/b/c", b"a/b/../b/c"),
			(b"a/b/c/..", b"a/b"),
			(b"a/..", b""),
			(b"/a/..", b"/"),
			(b"a/../..", b".."),
			(b"/a/../..", b"/.."),
			(b"a/b/c/./", b"a/b/c/"),
			(b"a/b/c/../", b"a/b/"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_eq!(a, b)
		}
	}

	#[test]
	fn ne() {
		let vectors: [(&[u8], &[u8]); 3] = [
			(b"a/b/c", b"a/b/c/"),
			(b"a/b/c/", b"a/b/c/."),
			(b"a/b/c/../", b"a/b"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_ne!(a, b)
		}
	}

	#[test]
	fn file_name() {
		let vectors: [(&[u8], Option<&[u8]>); 2] = [
			(b"//a/b/foo//bar/", None),
			(b"//a/b/foo//bar", Some(b"bar")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.file_name().map(|s| s.as_bytes()), expected)
		}
	}

	#[test]
	fn parent() {
		let vectors: [(&[u8], Option<&[u8]>); 11] = [
			(b"", None),
			(b"/", None),
			(b".", None),
			(b"//a/b/foo//bar", Some(b"//a/b/foo/")),
			(b"//a/b/foo//", Some(b"//a/b/foo/")),
			(b"//a/b/foo/", Some(b"//a/b/foo")),
			(b"//a/b/foo", Some(b"//a/b")),
			(b"//a/b", Some(b"//a")),
			(b"//a", Some(b"/./")),
			(b"/./", Some(b"/.")),
			(b"/.", Some(b"/")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.parent().map(Path::as_bytes), expected)
		}
	}

	#[test]
	fn suffix() {
		let vectors: [(&[u8], &[u8], Option<&[u8]>); 3] = [
			(b"/foo/bar/baz", b"/foo/bar", Some(b"baz")),
			(b"//foo", b"/", Some(b".//foo")),
			(b"/a/b/baz", b"/foo/bar", None),
		];

		for (path, prefix, expected_suffix) in vectors {
			let path = Path::new(path).unwrap();
			let suffix = path.suffix(Path::new(prefix).unwrap());
			assert_eq!(suffix.as_deref().map(Path::as_bytes), expected_suffix)
		}
	}
}
