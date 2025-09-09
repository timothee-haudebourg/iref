mod r#mut;
mod segment;

pub use r#mut::*;
pub use segment::*;

crate::common::path_impl!("IRI");

/// Parses a IRI [`Path`] at compile time.
#[macro_export]
macro_rules! ipath {
	($value:literal) => {
		match $crate::iri::Path::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI path"),
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
		let path = Path::new("a/b").unwrap();

		assert!(!path.is_empty());
		assert!(!path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "a");
		assert!(segments.next().unwrap().as_str() == "b");
		assert!(segments.next().is_none());
	}

	#[test]
	fn non_empty_absolute() {
		let path = Path::new("/foo/bar").unwrap();
		assert!(!path.is_empty());
		assert!(path.is_absolute());

		let mut segments = path.segments();
		assert!(segments.next().unwrap().as_str() == "foo");
		assert!(segments.next().unwrap().as_str() == "bar");
		assert!(segments.next().is_none());
	}

	#[test]
	fn next_segment() {
		let vectors = [
			("foo/bar", 0, Some(("foo", 4))),
			("foo/bar", 4, Some(("bar", 8))),
			("foo/bar", 8, None),
			("foo/bar/", 8, Some(("", 9))),
			("foo/bar/", 9, None),
			("//foo", 1, Some(("", 2))),
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
		let vectors = [
			("/foo/bar", 1, None),
			("foo/bar", 0, None),
			("foo/bar", 4, Some(("foo", 0))),
			("foo/bar", 8, Some(("bar", 4))),
			("foo/bar/", 8, Some(("bar", 4))),
			("foo/bar/", 9, Some(("", 8))),
			("//a/b", 4, Some(("a", 2))),
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
		let vectors = [
			("", None),
			("/", None),
			("//", Some("")),
			("/foo/bar", Some("foo")),
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
		let vectors: [(&str, &[&str]); 8] = [
			("", &[]),
			("foo", &["foo"]),
			("/foo", &["foo"]),
			("foo/", &["foo", ""]),
			("/foo/", &["foo", ""]),
			("a/b/c/d", &["a", "b", "c", "d"]),
			("a/b//c/d", &["a", "b", "", "c", "d"]),
			("//a/b/foo//bar/", &["", "a", "b", "foo", "", "bar", ""]),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected)
					.all(|(a, b)| a.as_str() == *b)
			)
		}
	}

	#[test]
	fn segments_rev() {
		let vectors: [(&str, &[&str]); 8] = [
			("", &[]),
			("foo", &["foo"]),
			("/foo", &["foo"]),
			("foo/", &["foo", ""]),
			("/foo/", &["foo", ""]),
			("a/b/c/d", &["a", "b", "c", "d"]),
			("a/b//c/d", &["a", "b", "", "c", "d"]),
			("//a/b/foo//bar/", &["", "a", "b", "foo", "", "bar", ""]),
		];

		for (input, expected) in vectors {
			let path = Path::new(input).unwrap();
			let segments: Vec<_> = path.segments().rev().collect();
			assert_eq!(segments.len(), expected.len());
			assert!(
				segments
					.into_iter()
					.zip(expected.into_iter().rev())
					.all(|(a, b)| a.as_str() == *b)
			)
		}
	}

	#[test]
	fn normalized() {
		let vectors = [
			("", ""),
			("a/b/c", "a/b/c"),
			("a/..", ""),
			("a/b/..", "a/"),
			("a/b/../", "a/"),
			("a/b/c/..", "a/b/"),
			("a/b/c/.", "a/b/c/"),
			("a/../..", "../"),
			("/a/../..", "/"), // Errata 4547 is only implemented for relative paths.
		];

		for (input, expected) in vectors {
			// eprintln!("{input}, {expected}");
			let path = Path::new(input).unwrap();
			let output = path.normalized();
			assert_eq!(output.as_str(), expected);
		}
	}

	#[test]
	fn eq() {
		let vectors = [
			("a/b/c", "a/b/c"),
			("a/b/c", "a/b/c/."),
			("a/b/c/", "a/b/c/./"),
			("a/b/c", "a/b/../b/c"),
			("a/b/c/..", "a/b"),
			("a/..", ""),
			("/a/..", "/"),
			("a/../..", ".."),
			("/a/../..", "/.."),
			("a/b/c/./", "a/b/c/"),
			("a/b/c/../", "a/b/"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_eq!(a, b)
		}
	}

	#[test]
	fn ne() {
		let vectors = [
			("a/b/c", "a/b/c/"),
			("a/b/c/", "a/b/c/."),
			("a/b/c/../", "a/b"),
		];

		for (a, b) in vectors {
			let a = Path::new(a).unwrap();
			let b = Path::new(b).unwrap();
			assert_ne!(a, b)
		}
	}

	#[test]
	fn file_name() {
		let vectors = [("//a/b/foo//bar/", None), ("//a/b/foo//bar", Some("bar"))];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.file_name().map(Segment::as_str), expected)
		}
	}

	#[test]
	fn parent() {
		let vectors = [
			("", None),
			("/", None),
			(".", None),
			("//a/b/foo//bar", Some("//a/b/foo/")),
			("//a/b/foo//", Some("//a/b/foo/")),
			("//a/b/foo/", Some("//a/b/foo")),
			("//a/b/foo", Some("//a/b")),
			("//a/b", Some("//a")),
			("//a", Some("/./")),
			("/./", Some("/.")),
			("/.", Some("/")),
		];

		for (input, expected) in vectors {
			let input = Path::new(input).unwrap();
			assert_eq!(input.parent().map(Path::as_str), expected)
		}
	}

	#[test]
	fn suffix() {
		let vectors = [
			("/foo/bar/baz", "/foo/bar", Some("baz")),
			("//foo", "/", Some(".//foo")),
			("/a/b/baz", "/foo/bar", None),
		];

		for (path, prefix, expected_suffix) in vectors {
			// eprintln!("{path} over {prefix} => {expected_suffix:?}");
			let path = Path::new(path).unwrap();
			let suffix = path.suffix(Path::new(prefix).unwrap());
			assert_eq!(suffix.as_deref().map(Path::as_str), expected_suffix)
		}
	}
}
