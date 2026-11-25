use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
// The only `unsafe` of the crate lives in this module; the library forbids it.
// SAFETY: see the comment on the single `fcntl(1, F_GETFD)` call in the module.
#[allow(unsafe_code)]
mod stdout_probe;

use efs_bill::{run, Options};

/// Offline estimate of AWS EFS Standard/IA GB-month cost from local JSON. Reads files only, no network.
#[derive(Parser, Debug)]
#[command(
    name = "efs-bill",
    version,
    about = "Offline estimate of AWS EFS Standard/IA GB-month cost from local JSON. Reads files only, no network.",
    after_help = "Exit codes: 0 ok, 2 input or usage error.\nPricing: https://aws.amazon.com/efs/pricing/ (geraadpleegd 2026-11-09)."
)]
struct Cli {
    /// JSON export files or a directory of *.json files.
    #[arg(required = true, value_name = "PATH")]
    inputs: Vec<PathBuf>,
    /// Price file. Default: built-in us-east-1 rates from the EFS pricing page (2026-11-09).
    #[arg(long, value_name = "FILE")]
    prices: Option<PathBuf>,
    /// Group rows by this Tags/Metadata key.
    #[arg(long, value_name = "KEY")]
    group_by: Option<String>,
    /// Print JSON (`schema_version` 1) instead of the text table.
    #[arg(long)]
    json: bool,
}

fn cannot_write() -> ExitCode {
    let _ = writeln!(std::io::stderr(), "efs-bill: could not write output");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    // Rust replaces a closed descriptor 1 with /dev/null before `main`; the probe
    // recorded before that whether it was closed.
    let stdout_closed = stdout_probe::stdout_was_closed_at_start();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            if stdout_closed && code == 0 {
                return cannot_write();
            }
            let _ = e.print();
            return ExitCode::from(code);
        }
    };
    match run(&Options {
        inputs: cli.inputs,
        prices: cli.prices,
        group_by: cli.group_by,
        json: cli.json,
    }) {
        Ok(text) => {
            if stdout_closed {
                return cannot_write();
            }
            let mut out = stdout().lock();
            if out.write_all(text.as_bytes()).is_err() || out.flush().is_err() {
                return cannot_write();
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "efs-bill: {err}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_builds() {
        crate::Cli::command().debug_assert();
    }

    #[test]
    fn help_mentions_flags() {
        let mut cmd = crate::Cli::command();
        let mut buf = Vec::new();
        cmd.write_long_help(&mut buf).unwrap();
        let help = String::from_utf8(buf).unwrap();
        assert!(help.contains("--group-by"));
        assert!(help.contains("--prices"));
        assert!(help.contains("efs/pricing"));
    }
}
