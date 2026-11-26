# Kwaliteit

pep: 10

Tak `main`. Debug en release.

- Datum: 2026-11-25. Geteste codecommit: c083573. Linux, cargo 1.99.0.
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --locked`: 72 geslaagd, 0 mislukt, 0 genegeerd (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0.
- `cargo test --release --locked`: 72 geslaagd, 0 mislukt, 0 genegeerd (zelfde verdeling). Exit 0.
- Versie 0.1.0. `rust-version` 1.85.
- Rust 1.85 (2026-11-25, cargo 1.85.1, rustc 1.85.1, gedraaid op checkout 948a878, code gelijk aan c083573): `cargo +1.85 fmt --check`: exit 0. `cargo +1.85 clippy --all-targets --locked -- -D warnings`: exit 0. `cargo +1.85 test --locked`: 72 geslaagd, 0 mislukt, 0 genegeerd (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0. `cargo +1.85 test --release --locked`: 72 geslaagd, 0 mislukt, 0 genegeerd (38 lib, 2 main, 6 claims, 26 cli, 0 doc). Exit 0. Gedraaid 2026-11-25.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport. Commits na c083573 raken alleen documenten.

## Niet uitgevoerd

Niet uitgevoerd (not run) in deze meting: cargo package, cargo deny, cargo audit, een GitHub Actions-run, fuzzing, mutatietests.

## Documenten

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` en `QUALITY.nl.md` bestaan. `task.md` en `blueprint.md` zijn lokale notities. Ze staan in `.gitignore` en niet in git.
