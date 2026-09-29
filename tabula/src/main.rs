//! The `tabula` command.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

use tabula::server::{Options, Server, VERSION};

/// The port Tabula prefers: Orange's edition year.
const DEFAULT_PORT: u16 = 2026;

const USAGE: &str = "\
Usage: tabula [OPTIONS] [WORKSPACE]

Opens Tabula, a writing table for Orange, on the folder WORKSPACE
(default: the current folder) in your web browser.

Options:
      --port <PORT>      Serve on this local port [default: 2026; 0 picks one]
      --orangec <PATH>   Use this orangec instead of searching for one
      --library <DIR>    Read The Orange Book and manuals from this Orange checkout
      --no-open          Print the link instead of opening a browser
  -h, --help             Print help
  -V, --version          Print version
";

struct Arguments {
    workspace: PathBuf,
    port: Option<u16>,
    orangec: Option<PathBuf>,
    library: Option<PathBuf>,
    open: bool,
}

enum Parsed {
    Run(Arguments),
    Help,
    Version,
}

fn parse(arguments: impl Iterator<Item = OsString>) -> Result<Parsed, String> {
    let mut workspace: Option<PathBuf> = None;
    let mut port = None;
    let mut orangec = None;
    let mut library = None;
    let mut open = true;
    let mut arguments = arguments.peekable();
    let mut options_done = false;
    while let Some(argument) = arguments.next() {
        let text = argument.to_str().unwrap_or("");
        if !options_done && text.starts_with('-') && text != "-" {
            match text {
                "-h" | "--help" => return Ok(Parsed::Help),
                "-V" | "--version" => return Ok(Parsed::Version),
                "--no-open" => open = false,
                "--" => options_done = true,
                "--port" => {
                    let value = arguments.next().ok_or("--port needs a value")?;
                    let value = value.to_str().ok_or("--port must be a number")?;
                    port = Some(
                        value
                            .parse::<u16>()
                            .map_err(|_| "--port must be a number from 0 to 65535")?,
                    );
                }
                "--orangec" => {
                    orangec = Some(PathBuf::from(
                        arguments.next().ok_or("--orangec needs a path")?,
                    ));
                }
                "--library" => {
                    library = Some(PathBuf::from(
                        arguments.next().ok_or("--library needs a folder")?,
                    ));
                }
                other => return Err(format!("unknown option `{}`", other.escape_default())),
            }
        } else if workspace.is_none() {
            workspace = Some(PathBuf::from(argument));
        } else {
            return Err("only one workspace folder may be given".to_owned());
        }
    }
    Ok(Parsed::Run(Arguments {
        workspace: workspace.unwrap_or_else(|| PathBuf::from(".")),
        port,
        orangec,
        library,
        open,
    }))
}

fn open_browser(url: &str) -> bool {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(windows) {
        let mut command = Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn main() -> ExitCode {
    let arguments = match parse(std::env::args_os().skip(1)) {
        Ok(Parsed::Run(arguments)) => arguments,
        Ok(Parsed::Help) => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Parsed::Version) => {
            println!("tabula {VERSION}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("tabula: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let server = match Server::bind(Options {
        workspace: arguments.workspace,
        port: arguments.port.unwrap_or(DEFAULT_PORT),
        port_fallback: arguments.port.is_none(),
        orangec: arguments.orangec,
        library: arguments.library,
        token: None,
    }) {
        Ok(server) => server,
        Err(message) => {
            eprintln!("tabula: {message}");
            return ExitCode::FAILURE;
        }
    };
    let state = server.state();
    let url = server.launch_url();
    println!("Tabula {VERSION}, a writing table for Orange");
    println!("  workspace  {}", state.workspace.root().display());
    match &state.orangec {
        Some(compiler) => println!(
            "  orangec    {} ({})",
            compiler.path().display(),
            compiler.version()
        ),
        None => println!("  orangec    not found; checking and evaluation are off (use --orangec)"),
    }
    match &state.library {
        Some(library) => println!("  library    {}", library.root().display()),
        None => println!("  library    not found; the Book and manuals are off (use --library)"),
    }
    println!("  open       {url}");
    if arguments.open && !open_browser(&url) {
        println!("  (could not open a browser; open the link above yourself)");
    }
    println!("Stop Tabula by closing this terminal or pressing Ctrl+C.");
    server.run();
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{Parsed, parse};
    use std::ffi::OsString;

    fn run(arguments: &[&str]) -> Result<Parsed, String> {
        parse(arguments.iter().map(OsString::from))
    }

    #[test]
    fn parses_options_and_workspace() {
        let Ok(Parsed::Run(parsed)) = run(&[
            "--port",
            "0",
            "--no-open",
            "--orangec",
            "/x/orangec",
            "work",
        ]) else {
            panic!("expected a run");
        };
        assert_eq!(parsed.port, Some(0));
        assert!(!parsed.open);
        assert_eq!(parsed.workspace, std::path::PathBuf::from("work"));
        assert_eq!(parsed.orangec, Some(std::path::PathBuf::from("/x/orangec")));
        let Ok(Parsed::Run(defaults)) = run(&[]) else {
            panic!("expected a run");
        };
        assert_eq!(defaults.workspace, std::path::PathBuf::from("."));
        assert!(defaults.open && defaults.port.is_none());
        assert!(matches!(run(&["-h"]), Ok(Parsed::Help)));
        assert!(matches!(run(&["--version"]), Ok(Parsed::Version)));
        let Ok(Parsed::Run(dashed)) = run(&["--", "--odd"]) else {
            panic!("expected a run");
        };
        assert_eq!(dashed.workspace, std::path::PathBuf::from("--odd"));
    }

    #[test]
    fn rejects_bad_arguments() {
        assert!(run(&["--port"]).is_err());
        assert!(run(&["--port", "70000"]).is_err());
        assert!(run(&["--bogus"]).is_err());
        assert!(run(&["a", "b"]).is_err());
    }
}
