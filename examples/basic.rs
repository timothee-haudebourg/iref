use iref::{Iri, IriError};

fn main() -> Result<(), IriError<&'static str>> {
	let iri = Iri::new("https://www.rust-lang.org/foo/bar?query#frag")?;

	println!("scheme: {}", iri.scheme());
	println!("authority: {}", iri.authority().unwrap());
	println!("path: {}", iri.path());
	println!("query: {}", iri.query().unwrap());
	println!("fragment: {}", iri.fragment().unwrap());

	Ok(())
}
