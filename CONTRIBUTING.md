# How to Contribute

Contributions are welcome! Here is how to contribute.

## Contribution Guidelines

Please read the [contribution guidelines](https://github.com/timothee-haudebourg/admin/blob/main/files/CONTRIBUTING.md) before submitting your changes. Contributions that do not adhere to these guidelines may be rejected without further explanation.

## Project architecture

```
src/
├── lib.rs       Main entry point and public API exports
├── common/      Shared utilities (scheme, port, path context, parsing)
├── uri/         URI types and components (primary implementation)
├── iri/         IRI types (auto-generated from uri/ by build.rs)
├── uri_iri/     Cross-conversions between URI and IRI types
└── url.rs       Optional integration with the `url` crate
```

Files are organised so that each URI/IRI component (authority, path, query,
fragment, etc.) is defined in its own module, which should make finding them
easy.

### Code generation

The `iri/` directory is **auto-generated** from `uri/` by `build.rs` via text
replacement (`URI` → `IRI`, `Uri` → `Iri`, etc.). This keeps a single source
of truth for the core logic: only `uri/` should be edited directly.

### Borrowed/owned duality

Every type follows a borrowed/owned pattern (e.g. `Uri`/`UriBuf`,
`Path`/`PathBuf`). These dual types are generated using the `str-newtype`
crate. Borrowed types are zero-copy and immutable; owned types (gated behind
`std`) allocate a single `String` buffer and support in-place mutation through
helper types like `PathMut` and `AuthorityMut`.

### Validation

Types are validated at parse time using ABNF grammars compiled to DFA state
machines by the `static-automata` crate. Each type is annotated with
`#[automaton(grammar::...)]` to tie it to the relevant grammar rule.
