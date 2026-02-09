mod cli;
mod output;
mod rfc;
mod skill;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, RfcCommand, SkillCommand};

fn main() {
    if let Err(error) = run() {
        output::print_error(format!("{error:#}"));
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
            SkillCommand::Init(init_args) => skill::init::run(init_args),
            SkillCommand::New(new_args) => skill::init::run_new(new_args),
            SkillCommand::Validate(validate_args) => skill::validate::run(validate_args),
            SkillCommand::List(list_args) => skill::list::run(list_args),
            SkillCommand::Dump(dump_args) => skill::dump::run(dump_args),
            SkillCommand::Install(install_args) => skill::install::run(install_args),
            SkillCommand::Export(export_args) => skill::export::run(export_args),
        },
    }
}
