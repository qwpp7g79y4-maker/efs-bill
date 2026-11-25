# Contributing

Thanks for helping. Please keep changes small and checked.

## Patches and tests

A patch should come with a test. A fix needs a test that fails without it. A
new feature needs a test that shows it works. Docs-only changes need no test.

Before you send a patch, run:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Where it goes

Patches come in through GitHub Issues. Open an issue, describe the problem, and
attach the patch (or a link to it). Please do not open a private message or an
unannounced pull request.

## Prices

Do not invent prices. A price change needs the AWS source URL and the date you
consulted it, and an entry in `CHANGELOG.md`.

## Security

Do not report vulnerabilities in a public issue. See `SECURITY.md`.

## License

By contributing you agree that your work is licensed as `MIT OR Apache-2.0`,
like the rest of the project.
