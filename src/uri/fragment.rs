crate::common::fragment!("URI");

/// Parses an URI [`Fragment`] at compile time.
#[macro_export]
macro_rules! fragment {
	($value:literal) => {
		match $crate::uri::Fragment::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI fragment"),
		}
	};
}
