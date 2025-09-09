crate::common::authority_mut!();

#[cfg(test)]
mod tests {
	use crate::Iri;

	#[test]
	fn explicit_empty_with_authority_alike_path() {
		let iri = Iri::new("scheme:////").unwrap();
		let authority = iri.authority();

		assert!(authority.unwrap().is_empty());
	}
}
