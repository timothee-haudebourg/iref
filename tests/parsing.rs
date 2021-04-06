extern crate iref;

use iref::Iri;

#[test]
fn test1() {
	let buffer = "https://www.rust-lang.org/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().unwrap(), "www.rust-lang.org");
	assert_eq!(iri.path(), "/foo/bar");
}

#[test]
fn test2() {
	let buffer = "https://[::]/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().unwrap(), "[::]");
	assert_eq!(iri.path(), "/foo/bar");
}

#[test]
fn test3() {
	let buffer = "https://[::192.128.0.1]/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().unwrap(), "[::192.128.0.1]");
	assert_eq!(iri.path(), "/foo/bar");
}

#[test]
#[should_panic]
fn test4() {
	let buffer = "https://[::256.128.0.1]/foo/bar#frag"; // 256.128.0.1 is not a valid IPv4
	Iri::new(buffer).expect("parsing failed");
}

#[test]
fn test5() {
	let buffer = "https:///foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().unwrap().is_empty());
	assert_eq!(iri.path(), "/foo/bar");
}

#[test]
fn test6() {
	let buffer = "https:/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().is_none());
	assert_eq!(iri.path(), "/foo/bar");
}

#[test]
fn test7() {
	let buffer = "https:foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().is_none());
	assert_eq!(iri.path(), "foo/bar");
}

#[test]
#[should_panic]
fn test8() {
	let buffer = "https:foo/bar space";
	let iri = Iri::new(buffer).unwrap();

	println!("{}", iri.path());
}

#[test]
fn test9() {
	let iri1 = Iri::new("https:foo/bar").unwrap();
	let iri2 = Iri::new("https:foo/%62%61%72").unwrap();

	assert_eq!(iri1, iri2)
}
