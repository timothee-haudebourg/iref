crate::common::host!("IRI");

/// Parses a IRI authority [`Host`] at compile time.
#[macro_export]
macro_rules! ihost {
	($value:literal) => {
		match $crate::iri::Host::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI authority host"),
		}
	};
}
