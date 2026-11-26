# efs-bill

## Why

Estimate monthly **Amazon EFS** Standard and Infrequent Access (IA) GB-month
cost from a local JSON export, and show a short savings line when the export
suggests moving Standard bytes to IA.

It reads files only. It uses no network and holds no AWS credentials.

The tests and the measured numbers are in `QUALITY.md`.

## Installation

Rust 1.85+. From a checkout:

```sh
cargo install --path .
```

## Quick start

```sh
efs-bill examples/standard.json
efs-bill examples/ia.json --json
efs-bill examples/standard.json examples/ia.json --group-by env
```

## Quickstart in 5 minutes

1. Install with `cargo install --path .` (see Installation).
2. Run `efs-bill examples/standard.json`. It prints the Standard cost of one
   file system.
3. Run `efs-bill examples/ia.json --json` for the same estimate as JSON.
4. Point it at your own export (a file or a directory of `*.json`).
5. Read Limitations and Prices and responsibility before you rely on a number.

The examples are only read. Nothing is written to `examples/`.

## Options

```text
efs-bill [OPTIONS] <PATH>...
```

- `<PATH>...`: JSON export files, or a directory of `*.json` files.
- `--prices FILE`: use your own price file instead of the built-in rates.
  The format is the one in `prices/us-east-1.json`.
- `--group-by KEY`: aggregate rows by this `Tags` or `Metadata` key.
- `--json`: print JSON (`schema_version` 1) instead of the text table.
- `--help`, `--version`.

## Example

```sh
efs-bill examples/ia.json
```

The report is a tab-separated table with one row per file system and a total
row: Standard and IA bytes, Standard and IA cost, total, and the saving if
Standard bytes moved to IA. For 1 GiB in IA, `examples/ia.json` gives an IA
cost of `0.016 USD`.

## How it counts

- Input: one or more JSON files, or a directory of `*.json`, in the shape
  `{ "FileSystems": [ { "FileSystemId", "Name"?, "StandardBytes"?, "IaBytes"?,
  "SizeInBytes"?, "LifecyclePolicies"?, "Tags"? } ] }` or a single filesystem.
  Fields follow
  [FileSystemDescription](https://docs.aws.amazon.com/efs/latest/ug/API_FileSystemDescription.html)
  (geraadpleegd 2026-11-09).
- **Standard:** `StandardBytes / 1024³ × $0.30` from
  [Amazon EFS pricing](https://aws.amazon.com/efs/pricing/) (geraadpleegd
  2026-11-09). AWS meters storage in binary GB (`1 GB = 2³⁰` bytes).
- **IA:** `IaBytes / 1024³ × $0.016`. That is the us-east-1 Regional
  (Multi-AZ) **Elastic Throughput** rate on the same page (price list
  published 2026-11-04). The legacy-throughput IA rate on that page is
  $0.025; pass `--prices` with your own file for that rate.
- `SizeInBytes.Value` / `ValueInIA` are accepted when explicit Standard/IA
  fields are absent.
- **IA savings tip:** when `LifecyclePolicies` (or `AccessPattern` /
  `SuggestIaBytes`) suggest cold Standard data, the text report prints a
  `# move … to IA → save …` line using Standard−IA rate difference.
- `--group-by KEY` aggregates by `Tags` or `Metadata`. A group shows one saving
  note, built from the whole group's saving.
- **Not AWS fields:** `StandardBytes`, `IaBytes`, `StorageBytes`, `AccessPattern`,
  `SuggestIaBytes` and `LifecyclePolicies` are not the fields of the AWS
  `FileSystemDescription`. They are the input shape of this tool. Real AWS has
  `SizeInBytes` and a separate `describe-lifecycle-configuration` call.
- **Lifecycle tip:** only a policy with a `TransitionToIA` rule (while Standard
  holds bytes and IA holds none) suggests moving the Standard bytes. A policy
  without that rule, such as `TransitionToPrimaryStorageClass` /
  `AFTER_1_ACCESS`, gives no advice.
- **Required size:** every file system needs at least one recognised size
  field (`StandardBytes`, `IaBytes`, `SizeInBytes.Value`/`ValueInIA` or
  `StorageBytes.Standard`/`IA`). Without one, or for an entry that is not a
  JSON object, the input is refused (exit 2) instead of counted as 0.
- A directory is read for files with the lowercase extension `.json` only. A
  directory without one is an error, not a bill of 0.
- **Price range:** each rate in a `--prices` file is 0 or more. A negative rate
  is refused (exit 2). If the IA rate is not below the Standard rate, no saving
  is claimed. A rate or size so large that the amount overflows gives exit 2.
- In the text table, a control character (newline, tab, escape) in a name,
  group or currency is shown as `?`. `--json` escapes them instead.
  The same holds for every error message on stderr.

## Exit codes

- `0`: the report was printed (also for `--help` and `--version`).
- `2`: input or usage error, for example a broken JSON file, a missing path,
  an input without a size field, a negative or overflowing rate, or an unknown
  option. The message goes to stderr.

## Prices and responsibility

The built-in rates are the us-east-1 Regional Elastic Throughput rates on the
[Amazon EFS pricing](https://aws.amazon.com/efs/pricing/) page, consulted
2026-11-09. They are in `prices/us-east-1.json` with their source and date.
This project does not invent prices. AWS can change its prices at any time, and
your bill can differ from this estimate. You are responsible for checking the
rates against your own AWS agreement before you act on a number. Use `--prices`
to supply your own file.

## Limitations

- Offline only: no AWS network client in v0.1.
- Free tier is not subtracted.
- Throughput, One Zone, and Archive classes are not modeled in v0.1.
  The built-in IA rate is Regional Elastic ($0.016), not legacy IA ($0.025).
- Each file is at most 32 MiB.

## Maintenance

Prices are a snapshot of 2026-11-09 and are not updated by themselves. When AWS
changes a rate, edit the price file (or pass your own with `--prices`) and note
the new source and date in `CHANGELOG.md`. Bugs and patches go through GitHub
Issues, see `CONTRIBUTING.md`. Security reports go to the address in
`SECURITY.md`.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The suite has 68 tests. See `QUALITY.md`.

## License

MIT OR Apache-2.0.

The licenses of the third-party crates built into the binary are listed in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).
