mod cli;
mod rfc;
mod skill;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, RfcCommand, SkillCommand};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Rfc(args) => match args.command {
            RfcCommand::Init => rfc::init::run(),
            RfcCommand::New(new_args) => rfc::create::create_rfc(&new_args),
            RfcCommand::Revise(revise_args) => rfc::revise::revise_rfc(&revise_args),
        },
        Command::Skill(args) => match args.command {
            SkillCommand::Init => skill::init::run(),
            SkillCommand::New(new_args) => skill::init::run_new(new_args),
            SkillCommand::Validate(validate_args) => skill::validate::run(validate_args),
        },
    }
}
