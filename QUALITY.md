# Quality

Branch `main`. Debug and release.

- Date: 2026-11-25. Tested code commit: ad9be4c. Linux, cargo 1.99.0.
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --locked`: 68 passed, 0 failed, 0 ignored (38 lib, 2 main, 6 claims, 22 cli, 0 doc). Exit 0.
- `cargo test --release --locked`: 68 passed, 0 failed, 0 ignored (same split). Exit 0.
- Tests: 68 passed, 0 failed, 0 ignored.
- Version 0.1.0. `rust-version` 1.85.

This page records checks. It is not a guarantee and it is not a fuzz report. Commits after ad9be4c touch only documents.

## Not run

Not run in this measurement: cargo package, cargo deny, cargo audit, a GitHub Actions run, fuzzing, mutation testing. `rust-version` 1.85 was not tried with a 1.85 toolchain.

## Documents

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` and `QUALITY.nl.md` exist. `task.md` and `blueprint.md` are local notes. They are in `.gitignore` and not in git.
