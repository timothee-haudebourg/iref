crate::common::path_mut_impl!("IRI");

#[cfg(test)]
mod tests {
	use super::*;
	use crate::iri::{PathBuf, Segment};

	#[test]
	fn push() {
		let vectors = [
			("", "foo", "foo"),
			("/", "foo", "/foo"),
			("", "", "./"),
			("/", "", "/./"),
			("foo", "bar", "foo/bar"),
			("/foo", "bar", "/foo/bar"),
			("foo", "", "foo/"),
			("foo/bar", "", "foo/bar/"),
			("foo/", "", "foo//"),
			("a/b/c", "d", "a/b/c/d"),
			("/a/b/c", "d", "/a/b/c/d"),
			("a/b/c/", "d", "a/b/c//d"),
		];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.push(segment);
			assert_eq!(path_mut.as_str(), expected)
		}
	}

	#[test]
	fn pop() {
		let vectors = [
			("", ".."),
			("/", "/"),
			("/..", "/../.."),
			("foo", ""),
			("foo/bar", "foo"),
			("foo/bar/", "foo/bar"),
		];

		for (path, expected) in vectors {
			let mut path = PathBuf::new(path.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.pop();
			assert_eq!(path_mut.as_str(), expected)
		}
	}

	#[test]
	fn normalized() {
		let vectors = [
			("", ""),
			("a/b/c", "a/b/c"),
			("a/..", ""),
			("a/b/..", "a"),
			("a/b/../", "a/"),
			("a/b/c/..", "a/b"),
			("a/b/c/.", "a/b/c"),
			("a/../..", ".."),
			("/a/../..", "/"),
		];

		for (input, expected) in vectors {
			let mut path = PathBuf::new(input.to_string()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.normalize();
			assert_eq!(path_mut.as_str(), expected);
		}
	}
}
