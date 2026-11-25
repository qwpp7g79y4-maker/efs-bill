# Quality

Branch `main`. Debug only.

- Tests: 68 passed, 0 failed, 0 ignored (`cargo test --locked --no-fail-fast`, 2026-11-24, debug, on commit d6cd0f0).
- Version 0.1.0. `rust-version` 1.85.
- Tested code commit: d6cd0f0. That is the last commit that is not a pure document. Commits after it touch only documents.

## What was not in the short note

This page records checks. It is not a guarantee and it is not a fuzz report.

## Run 2026-11-24

`cargo test --locked --no-fail-fast` on d6cd0f0: 68 passed (38 lib, 2 main, 6 claims, 22 cli, 0 doc), 0 failed.
`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings` also finished with exit 0 on that tree. The suite was run again on the tree with the documents of the next commit.
This round fixed B1 to B6 of the review of 212a7fc and the open blocker of the final review of ded346d (control characters in error messages on stderr). N1 to N14 are still open.
On the old source (212a7fc), 8 of the new cli tests fail; on d6cd0f0 they pass.

## Earlier runs

On 2026-11-18, on this machine, Linux, with cargo -j 2, these commands finished with exit 0: cargo fmt --all -- --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked, and cargo test --release --locked. Debug and release passed the same tests. 0 failed.

On 2026-11-23 the same test command passed 47 (before this round). The `cargo fmt --check` and `cargo clippy -p efs-bill --all-targets -- -D warnings` results date from 2026-11-18, and `cargo package --list` showed no `target/`. They were not run again on 2026-11-24.

## Not run

On 2026-11-24 only `cargo test` was run. Not run: cargo fmt --check, cargo clippy, cargo test --release, cargo package, cargo deny, cargo audit, a GitHub Actions run, fuzzing, mutation testing. The CI file lists deny and audit. Those jobs have not run on GitHub for this tree, because this tree is not pushed.

## Documents

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` and `QUALITY.nl.md` exist. `task.md` and `blueprint.md` are local notes. They are in `.gitignore` and not in git.
