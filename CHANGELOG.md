# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Only the current major version changes are kept in this file.
See older versions of this file for older changes.

## [4.0.0]

### Breaking changes

- Consolidated workspace into a single crate (removed `iref-core` and
  `iref-macros`).
- Generated the `iri` module from the `uri` module via `build.rs`.
- Added `try_*` methods (replacing some panicking APIs).
- Added `no_std` support (owned types gated behind `std`).
- Renamed `Path::EMPTY` to `Path::EMPTY_RELATIVE`.
- Replaced `From<Iri>` for `url::Url` with `TryFrom`.
- `PathMut`, `PathBuf`, and `AuthorityMut` methods now return `&mut Self`
  for chaining.

### Added

- `url` crate compatibility layer.
- `Host::is_ipv4`, `Host::is_ipv6`, `Host::is_ip_literal`, `Host::to_ipv4`,
  `Host::to_ipv6`.
- `HostBuf::from_ipv4`, `HostBuf::from_ipv6`.
- Non-mutating component methods such as `Uri::with_scheme`,
  `Uri::with_authority`, `Uri::with_query`, `Uri::with_fragment`.
- `UriRef::without_fragment`, `UriRef::without_query_and_fragment`.
- `Scheme::HTTP`, `Scheme::HTTPS` and other common scheme constants.
- `PathBuf::from_segments`, `impl FromIterator<&Segment> for PathBuf`.
- `impl ExactSizeIterator for Segments`.
- `Port::as_u16`, `Port::as_u32` and other port conversion methods.
- `Uri::authority_host`, `Uri::authority_port` and other direct authority
  component accessors.
- More `PartialEq`/`PartialOrd` implementations across URI/IRI types.
- `CONTRIBUTING.md` and `AGENTS.md`.

### Fixed

- Path normalization and reference resolution.
- `relative_to` for mismatched scheme/authority presence.
- Host parsing bug.
- `is_ipv4` now strictly validates dec-octets per RFC 3986.
- `to_ipv6` now handles embedded IPv4 suffixes.
- `PathMut::lazy_push` disambiguation when URI is not empty.

### Migrating from 3.x

- Replace `iref-core` and `iref-macros` dependencies with `iref` directly.
- `Path::EMPTY` is now `Path::EMPTY_RELATIVE`.
- `url::Url::from(iri)` is now `url::Url::try_from(iri)?`.
- Mutation methods on `PathMut`, `PathBuf`, and `AuthorityMut` now return
  `&mut Self`. Calls that previously discarded the return value still work,
  but you can now chain them (e.g. `path.push(a)?.push(b)?`).
- Disable the default `std` feature to use `iref` in `no_std` environments.
  Owned types (`UriBuf`, `IriBuf`, etc.) require `std`.
