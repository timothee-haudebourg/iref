pub trait FragmentImpl {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self;

	fn as_bytes(&self) -> &[u8];

	fn len(&self) -> usize {
		self.as_bytes().len()
	}
}
