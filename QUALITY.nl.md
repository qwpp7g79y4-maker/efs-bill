# Kwaliteit

Tak `main`. Debug en release.

- Datum: 2026-11-25. Geteste codecommit: ad9be4c. Linux, cargo 1.99.0.
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0.
- `cargo test --locked`: 68 geslaagd, 0 mislukt, 0 genegeerd (38 lib, 2 main, 6 claims, 22 cli, 0 doc). Exit 0.
- `cargo test --release --locked`: 68 geslaagd, 0 mislukt, 0 genegeerd (zelfde verdeling). Exit 0.
- Versie 0.1.0. `rust-version` 1.85.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport. Commits na ad9be4c raken alleen documenten.

## Niet uitgevoerd

Niet uitgevoerd (not run) in deze meting: cargo package, cargo deny, cargo audit, een GitHub Actions-run, fuzzing, mutatietests. `rust-version` 1.85 is niet geprobeerd met een 1.85-toolchain.

## Documenten

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` en `QUALITY.nl.md` bestaan. `task.md` en `blueprint.md` zijn lokale notities. Ze staan in `.gitignore` en niet in git.
