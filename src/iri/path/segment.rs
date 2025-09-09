crate::common::segment!("IRI");

/// Parses a IRI path [`Segment`] at compile time.
#[macro_export]
macro_rules! isegment {
	($value:literal) => {
		match $crate::iri::Segment::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI path segment"),
		}
	};
}
