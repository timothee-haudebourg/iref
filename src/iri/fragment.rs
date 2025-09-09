crate::common::fragment!("IRI");

/// Parses an IRI [`Fragment`] at compile time.
#[macro_export]
macro_rules! ifragment {
	($value:literal) => {
		match $crate::iri::Fragment::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI fragment"),
		}
	};
}
