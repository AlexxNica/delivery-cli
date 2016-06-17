use cli::value_of;
use clap::{Arg, App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "token";

#[derive(Debug)]
pub struct TokenClapOptions<'n> {
    pub server: &'n str,
    pub port: &'n str,
    pub ent: &'n str,
    pub user: &'n str,
    pub verify: bool,
}
impl<'n> Default for TokenClapOptions<'n> {
    fn default() -> Self {
        TokenClapOptions {
            server: "",
            port: "",
            ent: "",
            user: "",
            verify: false,
        }
    }
}

impl<'n> TokenClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        TokenClapOptions {
            server: value_of(&matches, "server"),
            port: value_of(&matches, "api-port"),
            ent: value_of(&matches, "ent"),
            user: value_of(&matches, "user"),
            verify: matches.is_present("verify"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Create a local API token")
        .args(&make_arg_vec![
            "-u --user=[user] 'User name for Delivery authentication'",
            "-e --ent=[ent] 'The enterprise in which the project lives'",
            "--verify 'Verify the Token has expired'",
            "-s --server=[server] 'The Delivery server address'"])
        .args_from_usage(
            "--api-port=[api-port] 'Port for Delivery server'")
}
