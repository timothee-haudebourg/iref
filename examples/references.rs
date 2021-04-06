extern crate iref;

use iref::{Iri, IriRef, IriRefBuf};
use std::convert::TryInto;

fn main() -> Result<(), iref::Error> {
	let mut iri_ref = IriRefBuf::default(); // an IRI reference can be empty.

	// An IRI reference with a scheme is a valid IRI.
	iri_ref.set_scheme(Some("https".try_into()?));
	let iri: Iri = iri_ref.as_iri()?;

	// An IRI can be safely converted into an IRI reference.
	let _iri_ref: IriRef = iri.into();

	Ok(())
}
