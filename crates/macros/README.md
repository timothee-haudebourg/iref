# Build IRI and IRI references at compile time

[![Crate informations](https://img.shields.io/crates/v/iref.svg?style=flat-square)](https://crates.io/crates/iref)
[![License](https://img.shields.io/crates/l/iref.svg?style=flat-square)](https://github.com/timothee-haudebourg/iref#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/iref)

<!-- cargo-rdme start -->

This is a companion crate for [`iref`][iref] providing macros to build
`'static` URI/IRIs and URI/IRI references at compile time.

[iref]: <https://github.com/timothee-haudebourg/iref>

### Basic usage

Use the `uri!` (resp. `iri!`) macro to build URI (resp. IRI) statically, and
the `uri_ref!` (resp `iri_ref!`) macro to build URI (resp. IRI) references
statically.

```rust
use iref::{Iri, IriRef};
use static_iref::{iri, iri_ref};

const IRI: &'static Iri = iri!("https://www.rust-lang.org/foo/bar#frag");
const IRI_REF: &'static IriRef = iri_ref!("/foo/bar#frag");
```

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
