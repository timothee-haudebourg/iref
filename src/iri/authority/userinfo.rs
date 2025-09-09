crate::common::user_info!("IRI");

/// Parses a IRI authority [`UserInfo`] at compile time.
#[macro_export]
macro_rules! iuser_info {
	($value:literal) => {
		match $crate::iri::UserInfo::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid IRI authority user info"),
		}
	};
}
