crate::common::user_info!("URI");

/// Parses a URI authority [`UserInfo`] at compile time.
#[macro_export]
macro_rules! user_info {
	($value:literal) => {
		match $crate::uri::UserInfo::from_str($value) {
			Ok(value) => value,
			Err(_) => panic!("invalid URI authority user info"),
		}
	};
}
