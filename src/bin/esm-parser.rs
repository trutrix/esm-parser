//! Elder Scrolls Mod format parser.

use esm_parser::prelude::*;

fn main() -> esm_parser::Result<()> {
    // parse args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file_path>", args[0]);
        return Ok(())
    }

    // parse file using guesser
    let mut parser = ESMParser2::file(&args[1])?;
    parser.parse_top_level()
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn zeta() {
        let mut cmd = Command::cargo_bin("esm-parser").unwrap();
        cmd.arg("data/Zeta.esm");
        cmd.assert().success();
    }
}
