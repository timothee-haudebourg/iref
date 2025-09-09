crate::common::query!("URI");

/// Parses an URI [`Query`] at compile time.
#[macro_export]
macro_rules! query {
	($value:literal) => {
		match $crate::uri::Query::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI query"),
		}
	};
}
