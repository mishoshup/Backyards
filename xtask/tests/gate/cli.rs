use clap::CommandFactory;
use xtask::command::Cli;

#[test]
fn command_definition_is_consistent() {
    Cli::command().debug_assert();
}
