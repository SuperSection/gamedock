use clap::CommandFactory;
use clap_complete::generate;
use crate::cli::{Cli, CompletionsArgs};
use std::io;

pub fn completions(args: CompletionsArgs) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "gamedock", &mut io::stdout());
    Ok(())
}
