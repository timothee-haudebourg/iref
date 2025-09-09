crate::common::query!("IRI");

/// Parses an IRI [`Query`] at compile time.
#[macro_export]
macro_rules! iquery {
	($value:literal) => {
		match $crate::iri::Query::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI query"),
		}
	};
}
