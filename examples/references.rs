use iref::{Iri, IriRef, IriRefBuf};

fn main() {
	let mut iri_ref = IriRefBuf::default(); // an IRI reference can be empty.

	// An IRI reference with a scheme is a valid IRI.
	iri_ref.set_scheme(Some("https".try_into().unwrap()));
	let iri: &Iri = iri_ref.as_iri().unwrap();

	// An IRI can be safely converted into an IRI reference.
	let _iri_ref: &IriRef = iri.into();
}
