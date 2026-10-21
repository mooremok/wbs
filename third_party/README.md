# Vendored dependencies

## winit 0.30.13 (patched)

Upstream: https://github.com/rust-windowing/winit (Apache-2.0 OR MIT)

Local patch (`src/platform_impl/macos/view.rs`): an input method's `insertText:`
commit that arrives without marked text — how Chinese input methods commit their
punctuation, which is never pre-edited — was dropped, and the key event then
carried the raw ASCII key character, so every Chinese punctuation mark typed as
its English one. The patch keeps that text and puts it on the key event.
Upstream issue: https://github.com/rust-windowing/winit/issues/3814 (open since
2024-07). Examples, docs and tests are stripped.

**Drop this directory and the `[patch.crates-io]` entry in the workspace
`Cargo.toml` when upstream winit fixes #3814.**
