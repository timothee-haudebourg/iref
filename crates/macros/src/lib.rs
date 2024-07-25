//! This is crate provides useful macros to build `'static` URI/IRIs and URI/IRI
//! references at compile time. It is re-exported by the [`iref`] crate when the
//! `macros` feature is enabled.
//!
//! [`iref`]: <https://github.com/timothee-haudebourg/iref>
//!
//! ## Basic usage
//!
//! Using the `iref` crate, enable the `macros` feature and
//! use the `uri!` (resp. `iri!`) macro to build URI (resp. IRI) statically, or
//! the `uri_ref!` (resp `iri_ref!`) macro to build URI (resp. IRI) references
//! statically.
//!
//! ```rust
//! # extern crate iref_core as iref;
//! # mod iref { pub use iref_core::{Iri, IriRef}; pub use iref_macros::{iri, iri_ref}; };
//! use iref::{Iri, IriRef, iri, iri_ref};
//!
//! const IRI: &'static Iri = iri!("https://www.rust-lang.org/foo/bar#frag");
//! const IRI_REF: &'static IriRef = iri_ref!("/foo/bar#frag");
//! ```
use iref_core::{IriBuf, IriRefBuf, UriBuf, UriRefBuf};
use proc_macro::TokenStream;
use quote::quote;

/// Build an URI with a `'static` lifetime at compile time.
///
/// This macro expects a single string literal token representing the URI.
#[proc_macro]
pub fn uri(tokens: TokenStream) -> TokenStream {
	match syn::parse::<syn::LitStr>(tokens) {
		Ok(lit) => match UriBuf::new(lit.value().into_bytes()) {
			Ok(uri) => {
				let value = uri.as_bytes();
				quote! {
					unsafe {
						::iref::Uri::new_unchecked(&[#(#value),*])
					}
				}
				.into()
			}
			Err(_) => produce_error("invalid URI"),
		},
		Err(e) => e.to_compile_error().into(),
	}
}

/// Build an URI reference with a `'static` lifetime at compile time.
///
/// This macro expects a single string literal token representing the URI reference.
#[proc_macro]
pub fn uri_ref(tokens: TokenStream) -> TokenStream {
	match syn::parse::<syn::LitStr>(tokens) {
		Ok(lit) => match UriRefBuf::new(lit.value().into_bytes()) {
			Ok(uri_ref) => {
				let value = uri_ref.as_bytes();
				quote! {
					unsafe {
						::iref::UriRef::new_unchecked(&[#(#value),*])
					}
				}
				.into()
			}
			Err(_) => produce_error("invalid URI reference"),
		},
		Err(e) => e.to_compile_error().into(),
	}
}

/// Build an IRI with a `'static` lifetime at compile time.
///
/// This macro expects a single string literal token representing the IRI.
#[proc_macro]
pub fn iri(tokens: TokenStream) -> TokenStream {
	match syn::parse::<syn::LitStr>(tokens) {
		Ok(lit) => match IriBuf::new(lit.value()) {
			Ok(iri) => {
				let value = iri.as_str();
				quote! {
					unsafe {
						::iref::Iri::new_unchecked(#value)
					}
				}
				.into()
			}
			Err(_) => produce_error("invalid IRI"),
		},
		Err(e) => e.to_compile_error().into(),
	}
}

/// Build an IRI reference with a `'static` lifetime at compile time.
///
/// This macro expects a single string literal token representing the IRI reference.
#[proc_macro]
pub fn iri_ref(tokens: TokenStream) -> TokenStream {
	match syn::parse::<syn::LitStr>(tokens) {
		Ok(lit) => match IriRefBuf::new(lit.value()) {
			Ok(iri_ref) => {
				let value = iri_ref.as_str();
				quote! {
					unsafe {
						::iref::IriRef::new_unchecked(#value)
					}
				}
				.into()
			}
			Err(_) => produce_error("invalid IRI reference"),
		},
		Err(e) => e.to_compile_error().into(),
	}
}

fn produce_error(msg: &str) -> TokenStream {
	format!("compile_error!(\"{}\")", msg).parse().unwrap()
}
