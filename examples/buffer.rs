use std::borrow::Cow;

use iref::{IriBuf, IriError};

fn main() -> Result<(), IriError<Cow<'static, str>>> {
	let mut iri = IriBuf::new("https://www.rust-lang.org".to_string())?;

	iri.authority_mut()
		.unwrap()
		.set_port(Some("40".try_into()?));
	iri.set_path("/foo".try_into()?);
	iri.path_mut().push("bar".try_into()?);
	iri.set_query(Some("query".try_into()?));
	iri.set_fragment(Some("fragment".try_into()?));

	assert_eq!(iri, "https://www.rust-lang.org:40/foo/bar?query#fragment");

	Ok(())
}
