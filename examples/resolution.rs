use std::borrow::Cow;

use iref::{Iri, IriError, IriRefBuf};

fn main() -> Result<(), IriError<Cow<'static, str>>> {
	let base_iri = Iri::new("http://a/b/c/d;p?q")?;
	let mut iri_ref = IriRefBuf::new("g;x=1/../y".to_string())?;

	// non mutating resolution.
	assert_eq!(iri_ref.resolved(base_iri), "http://a/b/c/y");

	// in-place resolution.
	iri_ref.resolve(base_iri);
	assert_eq!(iri_ref, "http://a/b/c/y");

	Ok(())
}
