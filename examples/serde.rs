use iref::{IriBuf, IriRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Foo {
	iri: IriBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Bar<'a> {
	#[serde(borrow)] // See <https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl>.
	iri_ref: &'a IriRef,
}

fn main() {
	let parsed: Foo = serde_json::from_str("{ \"iri\": \"https://example.org/foo\" }").unwrap();
	let bar: Bar = serde_json::from_str("{ \"iri_ref\": \"../bar\" }").unwrap();

	eprintln!("{:?}", parsed);
	eprintln!("{:?}", bar);
}
