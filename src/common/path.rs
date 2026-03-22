#[derive(Debug, Default, Clone, Copy)]
pub struct PathContext {
	pub has_scheme: bool,
	pub has_authority: bool,
}

impl PathContext {
	pub fn from_bytes(bytes: &[u8]) -> Self {
		Self {
			has_scheme: super::parse::find_scheme(bytes, 0).is_some(),
			has_authority: super::parse::find_authority(bytes, 0).is_ok(),
		}
	}
}
