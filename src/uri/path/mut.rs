crate::common::path_mut_impl!("URI");

#[cfg(test)]
mod tests {
	use super::*;
	use crate::uri::{PathBuf, Segment};

	#[test]
	fn push() {
		let vectors: [(&[u8], &[u8], &[u8]); 12] = [
			(b"", b"foo", b"foo"),
			(b"/", b"foo", b"/foo"),
			(b"", b"", b"./"),
			(b"/", b"", b"/./"),
			(b"foo", b"bar", b"foo/bar"),
			(b"/foo", b"bar", b"/foo/bar"),
			(b"foo", b"", b"foo/"),
			(b"foo/bar", b"", b"foo/bar/"),
			(b"foo/", b"", b"foo//"),
			(b"a/b/c", b"d", b"a/b/c/d"),
			(b"/a/b/c", b"d", b"/a/b/c/d"),
			(b"a/b/c/", b"d", b"a/b/c//d"),
		];

		for (path, segment, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			let segment = Segment::new(&segment).unwrap();
			path_mut.lazy_push(segment);
			assert_eq!(path_mut.as_bytes(), expected)
		}
	}

	#[test]
	fn pop() {
		let vectors: [(&[u8], &[u8]); 6] = [
			(b"", b".."),
			(b"/", b"/"),
			(b"/..", b"/../.."),
			(b"foo", b""),
			(b"foo/bar", b"foo"),
			(b"foo/bar/", b"foo/bar"),
		];

		for (path, expected) in vectors {
			let mut path = PathBuf::new(path.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.pop();
			assert_eq!(path_mut.as_bytes(), expected)
		}
	}

	#[test]
	fn normalized() {
		let vectors: [(&[u8], &[u8]); 9] = [
			(b"", b""),
			(b"a/b/c", b"a/b/c"),
			(b"a/..", b""),
			(b"a/b/..", b"a"),
			(b"a/b/../", b"a/"),
			(b"a/b/c/..", b"a/b"),
			(b"a/b/c/.", b"a/b/c"),
			(b"a/../..", b".."),
			(b"/a/../..", b"/"),
		];

		for (input, expected) in vectors {
			let mut path = PathBuf::new(input.to_vec()).unwrap();
			let mut path_mut = PathMut::from_path(&mut path);
			path_mut.normalize();
			assert_eq!(path_mut.as_bytes(), expected);
		}
	}
}
