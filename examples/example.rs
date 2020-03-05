extern crate iri;

use iri::Iri;

fn main() -> Result<(), iri::Error> {
    let buffer = "https://www.rust-lang.org/";
	let iri = Iri::new(buffer)?;

	println!("scheme: {}", iri.scheme());

    Ok(())
}
