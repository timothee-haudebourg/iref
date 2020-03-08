#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate iri;

use iri::Iri;

#[test]
fn test1() {
    let buffer = "https://www.rust-lang.org/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().expect("no authority"), "www.rust-lang.org");
	assert_eq!(iri.path(), Some("/foo/bar"));
}

#[test]
fn test2() {
	let buffer = "https://[::]/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().expect("no authority"), "[::]");
	assert_eq!(iri.path(), Some("/foo/bar"));
}

#[test]
fn test3() {
	let buffer = "https://[::192.128.0.1]/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert_eq!(iri.authority().expect("no authority"), "[::192.128.0.1]");
	assert_eq!(iri.path(), Some("/foo/bar"));
}

#[test]
#[should_panic]
fn test4() {
	let buffer = "https://[::256.128.0.1]/foo/bar#frag"; // 256.128.0.1 is not a valid IPv4
	let iri = Iri::new(buffer).expect("parsing failed");
}

#[test]
fn test5() {
	let buffer = "https:///foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().is_none());
	assert_eq!(iri.path(), Some("/foo/bar"));
}

#[test]
fn test6() {
	let buffer = "https:/foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().is_none());
	assert_eq!(iri.path(), Some("/foo/bar"));
}

#[test]
fn test7() {
	let buffer = "https:foo/bar#frag";
	let iri = Iri::new(buffer).expect("parsing failed");

	assert_eq!(iri.scheme(), "https");
	assert!(iri.authority().is_none());
	assert_eq!(iri.path(), Some("foo/bar"));
}
