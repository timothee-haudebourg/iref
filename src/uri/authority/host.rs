crate::common::host!("URI");

/// Parses a URI authority [`Host`] at compile time.
#[macro_export]
macro_rules! host {
	($value:literal) => {
		match $crate::uri::Host::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI authority host"),
		}
	};
}
