#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate iri;

use iri::{Iri, IriBuf};

fn main() -> Result<(), iri::Error> {
	// init logger.
    stderrlog::new().verbosity(10).init().unwrap();

    let buffer = "https://www.rust-lang.org/foo/bar?query#frag";
	let iri = Iri::new(buffer)?;

	println!("IRI: {}", iri.as_str());
	println!("scheme: {}", iri.scheme());
	println!("authority: {}", iri.authority());
	println!("path: {}", iri.path().unwrap());
	println!("query: {}", iri.query().unwrap());
	println!("fragment: {}", iri.fragment().unwrap());

	let mut iri = IriBuf::new("https://www.rust-lang.org/foo/bar")?;

	iri.set_scheme("scheme")?;
	iri.set_authority("haudebourg.net")?;
	iri.authority_mut().set_userinfo(None);
	iri.authority_mut().set_port(Some("42"));
	iri.set_path("/1/2")?;
	iri.set_query(Some("foo=bar&hello=world"))?;
	iri.set_fragment(Some("ninja"));

	println!("IRI: {}", iri.as_str());
	println!("scheme: {}", iri.scheme());
	println!("authority: {} (host: {})", iri.authority(), iri.authority().host());
	println!("path: {}", iri.path().unwrap());
	println!("query: {}", iri.query().unwrap());
	println!("fragment: {}", iri.fragment().unwrap());

    Ok(())
}
