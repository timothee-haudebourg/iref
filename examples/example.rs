#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate iri;

use iri::Iri;

fn main() -> Result<(), iri::Error> {
	// init logger.
    stderrlog::new().verbosity(10).init().unwrap();

    let buffer = "https://www.rust-lang.org/foo/bar?query#frag";
	let iri = Iri::new(buffer)?;

	println!("scheme: {}", iri.scheme());
	println!("authority: {}", iri.authority().unwrap());
	println!("path: {}", iri.path().unwrap());
	println!("query: {}", iri.query().unwrap());
	println!("fragment: {}", iri.fragment().unwrap());

    Ok(())
}
