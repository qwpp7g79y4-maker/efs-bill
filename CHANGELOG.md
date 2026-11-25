# Changelog

## Unreleased

- Error messages on stderr: control characters from file values (escape, bell, newline) are shown as `?`.
- Text table: control characters in name, group and currency are shown as `?`.
- Overflowing amounts give exit 2 instead of a panic (checked decimal math).
- Negative rates are refused (exit 2); no saving is claimed when IA is not cheaper than Standard.
- A group shows one saving note built from the whole group.
- Lifecycle tip only for a policy with `TransitionToIA`; README says which fields are not AWS fields.
- Input without a recognised size field, or a non-object entry, is refused (exit 2). A directory without a lowercase `.json` file is an error.

## 0.1.0 — 2026-11-09

- First release: offline `efs-bill` from local EFS JSON exports.
- Standard ($0.30) and IA ($0.016) GB-month rates for us-east-1 Regional Elastic Throughput.
- IA savings tip when lifecycle / access hints are present.
- `--group-by`, `--prices`, `--json`.
