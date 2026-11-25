# Kwaliteit

Tak `main`. Alleen debug.

- Tests: 68 geslaagd, 0 mislukt, 0 genegeerd (`cargo test --locked --no-fail-fast`, 2026-11-24, debug, op commit d6cd0f0).
- Versie 0.1.0. `rust-version` 1.85.
- Geteste codecommit: d6cd0f0. Dat is de laatste commit die geen puur document is. Commits daarna raken alleen documenten.

## Wat niet in de korte notitie stond

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport.

## Run 2026-11-24

`cargo test --locked --no-fail-fast` op d6cd0f0: 68 geslaagd (38 lib, 2 main, 6 claims, 22 cli, 0 doc), 0 mislukt.
`cargo fmt --check` en `cargo clippy --all-targets --locked -- -D warnings` gaven op die boom ook exit 0. De suite is nog eens gedraaid op de boom met de documenten van de volgende commit.
Deze ronde herstelde B1 tot en met B6 van de review van 212a7fc en het open blokkerende punt van de eindreview van ded346d (besturingstekens in foutmeldingen op stderr). N1 tot en met N14 staan nog open.
Op de oude broncode (212a7fc) falen 8 van de nieuwe cli-tests; op d6cd0f0 slagen ze.

## Eerdere runs

Op 2026-11-18 liepen op deze machine, Linux, met cargo -j 2, deze commando's af met exit 0: cargo fmt --all -- --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo test --release --locked. Debug en release haalden dezelfde tests. 0 mislukt.

Op 2026-11-23 haalde hetzelfde testcommando 47 (vóór deze ronde). De uitslagen van `cargo fmt --check` en `cargo clippy -p efs-bill --all-targets -- -D warnings` dateren van 2026-11-18, en `cargo package --list` toonde geen `target/`. Ze zijn op 2026-11-24 niet opnieuw gedraaid.

## Niet uitgevoerd

Op 2026-11-24 is alleen `cargo test` gedraaid. Niet uitgevoerd: cargo fmt --check, cargo clippy, cargo test --release, cargo package, cargo deny, cargo audit, een GitHub Actions-run, fuzzing, mutatietests. Het CI-bestand noemt deny en audit. Die jobs hebben op GitHub niet gedraaid voor deze boom, want die is niet gepusht.

## Documenten

`README.md`, `README.nl.md`, `CONTRIBUTING.md`, `SECURITY.md` en `QUALITY.nl.md` bestaan. `task.md` en `blueprint.md` zijn lokale notities. Ze staan in `.gitignore` en niet in git.
