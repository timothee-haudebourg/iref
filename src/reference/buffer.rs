
/// Owned IRI-reference.
///
/// Holds a mutable buffer representing an IRI-reference.
/// This type can be used to create and/or modify existing IRI-references.
/// The authority and path can be accessed mutabily to modify their sub-components.
///
/// ## Example
///
/// ```
/// # use std::convert::TryInto;
/// # use iref::IriBuf;
/// # fn main() -> Result<(), iref::Error> {
/// let mut iri = IriBuf::new("https://www.rust-lang.org")?;
///
/// iri.authority_mut().unwrap().set_port(Some("40".try_into()?));
/// iri.set_path("/foo".try_into()?);
/// iri.path_mut().push("bar".try_into()?);
/// iri.set_query(Some("query".try_into()?));
/// iri.set_fragment(Some("fragment".try_into()?));
///
/// assert_eq!(iri, "https://www.rust-lang.org:40/foo/bar?query#fragment");
/// # Ok(())
/// # }
/// ```
///
/// See the [`IriRef`] type for more information about IRI-references.
#[derive(Default, Clone)]
pub struct IriRefBuf {
	pub(crate) p: ParsedIriRef,
	pub(crate) data: Vec<u8>,
}

impl IriRefBuf {
	/// Creates a new IRI reference by parsing and the input buffer.
	#[inline]
	pub fn from_vec(buffer: Vec<u8>) -> Result<IriRefBuf, (Error, Vec<u8>)> {
		match ParsedIriRef::new(&buffer) {
			Ok(p) => Ok(IriRefBuf { data: buffer, p }),
			Err(e) => Err((e, buffer)),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{Iri, IriRef, IriRefBuf};

	#[test]
	fn disambiguate1() {
		let mut iri_ref = IriRefBuf::new("scheme:a:b/c").unwrap();
		iri_ref.set_scheme(None);
		assert_eq!(iri_ref.as_str(), "./a:b/c")
	}

	#[test]
	fn disambiguate2() {
		let mut iri_ref = IriRefBuf::new("//host//path").unwrap();
		iri_ref.set_authority(None);
		assert_eq!(iri_ref.as_str(), "/.//path")
	}

	#[test]
	fn unambiguous_resolution() {
		let base_iri = Iri::new("http:/a/b").unwrap();

		let tests = [("../..//", "http:/.//")];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_normal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("g:h", "g:h"),
			("g", "http://a/b/c/g"),
			("./g", "http://a/b/c/g"),
			("g/", "http://a/b/c/g/"),
			("/g", "http://a/g"),
			("//g", "http://g"),
			("?y", "http://a/b/c/d;p?y"),
			("g?y", "http://a/b/c/g?y"),
			("#s", "http://a/b/c/d;p?q#s"),
			("g#s", "http://a/b/c/g#s"),
			("g?y#s", "http://a/b/c/g?y#s"),
			(";x", "http://a/b/c/;x"),
			("g;x", "http://a/b/c/g;x"),
			("g;x?y#s", "http://a/b/c/g;x?y#s"),
			("", "http://a/b/c/d;p?q"),
			(".", "http://a/b/c/"),
			("./", "http://a/b/c/"),
			("..", "http://a/b/"),
			("../", "http://a/b/"),
			("../g", "http://a/b/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn resolution_abnormal() {
		// https://www.w3.org/2004/04/uri-rel-test.html
		// NOTE we implement [Errata 4547](https://www.rfc-editor.org/errata/eid4547)
		let base_iri = Iri::new("http://a/b/c/d;p?q").unwrap();

		let tests = [
			("../../../g", "http://a/../g"), // NOTE without Errata 4547: "http://a/g"
			("../../../../g", "http://a/../../g"), // NOTE without Errata 4547: "http://a/g"
			("/./g", "http://a/g"),
			("/../g", "http://a/../g"), // NOTE without Errata 4547: "http://a/g"
			("g.", "http://a/b/c/g."),
			(".g", "http://a/b/c/.g"),
			("g..", "http://a/b/c/g.."),
			("..g", "http://a/b/c/..g"),
			("./../g", "http://a/b/g"),
			("./g/.", "http://a/b/c/g/"),
			("g/./h", "http://a/b/c/g/h"),
			("g/../h", "http://a/b/c/h"),
			("g;x=1/./y", "http://a/b/c/g;x=1/y"),
			("g;x=1/../y", "http://a/b/c/y"),
			("g?y/./x", "http://a/b/c/g?y/./x"),
			("g?y/../x", "http://a/b/c/g?y/../x"),
			("g#s/./x", "http://a/b/c/g#s/./x"),
			("g#s/../x", "http://a/b/c/g#s/../x"),
			("http:g", "http:g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			assert_eq!(IriRef::new(relative).unwrap().resolved(base_iri), *absolute);
		}
	}

	#[test]
	fn more_resolutions1() {
		let base_iri = Iri::new("http://a/bb/ccc/d;p?q").unwrap();

		let tests = [
			("#s", "http://a/bb/ccc/d;p?q#s"),
			("", "http://a/bb/ccc/d;p?q"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions2() {
		let base_iri = Iri::new("http://a/bb/ccc/./d;p?q").unwrap();

		let tests = [
			("..", "http://a/bb/"),
			("../", "http://a/bb/"),
			("../g", "http://a/bb/g"),
			("../..", "http://a/"),
			("../../", "http://a/"),
			("../../g", "http://a/g"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}

	#[test]
	fn more_resolutions3() {
		let base_iri = Iri::new("http://ab//de//ghi").unwrap();

		let tests = [
			("xyz", "http://ab//de//xyz"),
			("./xyz", "http://ab//de//xyz"),
			("../xyz", "http://ab//de/xyz"),
		];

		for (relative, absolute) in &tests {
			// println!("{} => {}", relative, absolute);
			let buffer: crate::IriBuf = IriRef::new(relative).unwrap().resolved(base_iri);
			assert_eq!(buffer.as_str(), *absolute);
		}
	}
}
