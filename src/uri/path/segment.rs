crate::common::segment!("URI");

/// Parses a URI path [`Segment`] at compile time.
#[macro_export]
macro_rules! segment {
	($value:literal) => {
		match $crate::uri::Segment::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI path segment"),
		}
	};
}
