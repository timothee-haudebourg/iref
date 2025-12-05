use super::{InvalidHost, InvalidPort, InvalidUserInfo};

crate::common::authority_mut!();

#[cfg(test)]
mod tests {
	use crate::Uri;

	#[test]
	fn explicit_empty_with_authority_alike_path() {
		let uri = Uri::new("scheme:////").unwrap();
		let authority = uri.authority();

		assert!(authority.unwrap().is_empty());
	}
}
