# Quality

pep: 8.6

Branch `main`. Debug and release.

- Date: 2026-11-25. Tested code commit: c083573. Linux, cargo 1.99.0.
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --locked`: 72 passed, 0 failed, 0 ignored (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0.
- `cargo test --release --locked`: 72 passed, 0 failed, 0 ignored (same split). Exit 0.
- Tests: 72 passed, 0 failed, 0 ignored.
- Version 0.1.0. `rust-version` 1.85.
- Rust 1.85 (2026-11-25, cargo 1.85.1, rustc 1.85.1, run on checkout 948a878, code identical to c083573): `cargo +1.85 fmt --check`: exit 0. `cargo +1.85 clippy --all-targets --locked -- -D warnings`: exit 0. `cargo +1.85 test --locked`: 72 passed, 0 failed, 0 ignored (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0. `cargo +1.85 test --release --locked`: 72 passed, 0 failed, 0 ignored (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0. Run 2026-11-25.

This page records checks. It is not a guarantee and it is not a fuzz report. Commits after c083573 touch only documents.

## Not run

Not run in this measurement: cargo package, cargo deny, cargo audit, a GitHub Actions run, fuzzing, mutation testing.

## Documents

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` and `QUALITY.nl.md` exist. `task.md` and `blueprint.md` are local notes. They are in `.gitignore` and not in git.
