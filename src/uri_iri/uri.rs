use crate::{InvalidIri, InvalidUri, Iri, IriRef, Uri, UriRef};

#[cfg(feature = "std")]
use crate::{IriBuf, IriRefBuf, UriBuf, UriRefBuf};

impl Uri {
	pub fn as_iri(&self) -> &Iri {
		unsafe { Iri::new_unchecked(self.as_str()) }
	}

	pub fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(self.as_str()) }
	}
}

#[cfg(feature = "std")]
impl UriBuf {
	pub fn into_iri(self) -> IriBuf {
		unsafe { IriBuf::new_unchecked(self.into_bytes()) }
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(self.into_bytes()) }
	}
}

#[cfg(feature = "std")]
impl AsRef<Iri> for UriBuf {
	fn as_ref(&self) -> &Iri {
		self.as_iri()
	}
}

#[cfg(feature = "std")]
impl AsRef<IriRef> for UriBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

#[cfg(feature = "std")]
impl std::borrow::Borrow<Iri> for UriBuf {
	fn borrow(&self) -> &Iri {
		self.as_iri()
	}
}

#[cfg(feature = "std")]
impl std::borrow::Borrow<IriRef> for UriBuf {
	fn borrow(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl UriRef {
	#[inline]
	pub fn as_iri(&self) -> Option<&Iri> {
		if self.scheme().is_some() {
			Some(unsafe { Iri::new_unchecked(self.as_str()) })
		} else {
			None
		}
	}

	#[inline]
	pub const fn as_iri_ref(&self) -> &IriRef {
		unsafe { IriRef::new_unchecked(self.as_str()) }
	}
}

impl AsRef<IriRef> for UriRef {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

impl<'a> From<&'a UriRef> for &'a IriRef {
	fn from(value: &'a UriRef) -> Self {
		value.as_iri_ref()
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Uri {
	type Error = InvalidUri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_uri().ok_or(InvalidUri(value))
	}
}

impl<'a> TryFrom<&'a UriRef> for &'a Iri {
	type Error = InvalidIri<&'a UriRef>;

	fn try_from(value: &'a UriRef) -> Result<Self, Self::Error> {
		value.as_iri().ok_or(InvalidIri(value))
	}
}

#[cfg(feature = "std")]
impl UriRefBuf {
	pub fn into_iri_ref(self) -> IriRefBuf {
		unsafe { IriRefBuf::new_unchecked(self) }
	}

	pub fn try_into_iri(self) -> Result<IriBuf, InvalidIri<Self>> {
		if self.scheme().is_some() {
			unsafe { Ok(IriBuf::new_unchecked(self.into_bytes())) }
		} else {
			Err(InvalidIri(self))
		}
	}
}

#[cfg(feature = "std")]
impl AsRef<IriRef> for UriRefBuf {
	fn as_ref(&self) -> &IriRef {
		self.as_iri_ref()
	}
}

#[cfg(feature = "std")]
impl From<UriRefBuf> for IriRefBuf {
	fn from(value: UriRefBuf) -> Self {
		value.into_iri_ref()
	}
}

#[cfg(feature = "std")]
impl TryFrom<UriRefBuf> for IriBuf {
	type Error = InvalidIri<UriRefBuf>;

	fn try_from(value: UriRefBuf) -> Result<Self, Self::Error> {
		value.try_into_iri()
	}
}

// Cross IRI/URI PartialEq and PartialOrd implementations.
//
// Since every valid URI is a valid IRI, comparisons convert both sides to
// `&IriRef` and delegate.

/// Generates bidirectional `PartialEq` and `PartialOrd` impls between two
/// types by converting each to `&IriRef`.
macro_rules! cross_cmp {
	($A:ty, $a_to_iri_ref:expr, $B:ty, $b_to_iri_ref:expr) => {
		impl PartialEq<$B> for $A {
			fn eq(&self, other: &$B) -> bool {
				fn inner<'a>(a: &'a $A, b: &'a $B) -> bool {
					let a: &'a IriRef = ($a_to_iri_ref)(a);
					let b: &'a IriRef = ($b_to_iri_ref)(b);
					*a == *b
				}

				inner(self, other)
			}
		}

		impl PartialEq<$A> for $B {
			fn eq(&self, other: &$A) -> bool {
				fn inner<'a>(a: &'a $B, b: &'a $A) -> bool {
					let a: &'a IriRef = ($b_to_iri_ref)(a);
					let b: &'a IriRef = ($a_to_iri_ref)(b);
					*a == *b
				}

				inner(self, other)
			}
		}

		impl PartialOrd<$B> for $A {
			fn partial_cmp(&self, other: &$B) -> Option<core::cmp::Ordering> {
				fn inner<'a>(a: &'a $A, b: &'a $B) -> Option<core::cmp::Ordering> {
					let a: &'a IriRef = ($a_to_iri_ref)(a);
					let b: &'a IriRef = ($b_to_iri_ref)(b);
					a.partial_cmp(b)
				}

				inner(self, other)
			}
		}

		impl PartialOrd<$A> for $B {
			fn partial_cmp(&self, other: &$A) -> Option<core::cmp::Ordering> {
				fn inner<'a>(a: &'a $B, b: &'a $A) -> Option<core::cmp::Ordering> {
					let a: &'a IriRef = ($b_to_iri_ref)(a);
					let b: &'a IriRef = ($a_to_iri_ref)(b);
					a.partial_cmp(b)
				}

				inner(self, other)
			}
		}
	};
}

// Borrowed <-> Borrowed
cross_cmp!(Uri, Uri::as_iri_ref, Iri, Iri::as_iri_ref);
cross_cmp!(Uri, Uri::as_iri_ref, IriRef, |x: &'a IriRef| x);
cross_cmp!(UriRef, UriRef::as_iri_ref, Iri, Iri::as_iri_ref);
cross_cmp!(UriRef, UriRef::as_iri_ref, IriRef, |x: &'a IriRef| x);

// Owned <-> Owned
#[cfg(feature = "std")]
cross_cmp!(
	UriBuf,
	|x: &'a UriBuf| x.as_iri_ref(),
	IriBuf,
	|x: &'a IriBuf| x.as_iri_ref()
);
#[cfg(feature = "std")]
cross_cmp!(
	UriBuf,
	|x: &'a UriBuf| x.as_iri_ref(),
	IriRefBuf,
	|x: &'a IriRefBuf| x.as_ref()
);
#[cfg(feature = "std")]
cross_cmp!(
	UriRefBuf,
	|x: &'a UriRefBuf| x.as_iri_ref(),
	IriBuf,
	|x: &'a IriBuf| x.as_iri_ref()
);
#[cfg(feature = "std")]
cross_cmp!(
	UriRefBuf,
	|x: &'a UriRefBuf| x.as_iri_ref(),
	IriRefBuf,
	|x: &'a IriRefBuf| x.as_ref()
);

// Borrowed <-> Owned
#[cfg(feature = "std")]
cross_cmp!(UriBuf, |x: &'a UriBuf| x.as_iri_ref(), Iri, Iri::as_iri_ref);
#[cfg(feature = "std")]
cross_cmp!(
	UriBuf,
	|x: &'a UriBuf| x.as_iri_ref(),
	IriRef,
	|x: &'a IriRef| x
);
#[cfg(feature = "std")]
cross_cmp!(
	UriRefBuf,
	|x: &'a UriRefBuf| x.as_iri_ref(),
	Iri,
	Iri::as_iri_ref
);
#[cfg(feature = "std")]
cross_cmp!(
	UriRefBuf,
	|x: &'a UriRefBuf| x.as_iri_ref(),
	IriRef,
	|x: &'a IriRef| x
);
#[cfg(feature = "std")]
cross_cmp!(IriBuf, |x: &'a IriBuf| x.as_iri_ref(), Uri, Uri::as_iri_ref);
#[cfg(feature = "std")]
cross_cmp!(
	IriBuf,
	|x: &'a IriBuf| x.as_iri_ref(),
	UriRef,
	UriRef::as_iri_ref
);
#[cfg(feature = "std")]
cross_cmp!(
	IriRefBuf,
	|x: &'a IriRefBuf| x.as_ref(),
	Uri,
	Uri::as_iri_ref
);
#[cfg(feature = "std")]
cross_cmp!(
	IriRefBuf,
	|x: &'a IriRefBuf| x.as_ref(),
	UriRef,
	UriRef::as_iri_ref
);
