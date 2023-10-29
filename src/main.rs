use anyhow::Result;
use clap::{Parser, Subcommand};
use reqq::{Reqq, ReqqOpts};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "reqq", version = "0.3.0", author = "Seth Etter <sethetter@gmail.com>", about = "Like insomnia or postman, but a CLI.", long_about = None)]
struct Args {
    /// The name of the request to execute.
    // TODO: this should be required if a subcommand isn't specified.
    request_name: Option<String>,

    /// The environment file to load.
    #[arg(short = 'e', long = "env", default_value = "default")]
    env: String,

    /// The directory containing the reqq files.
    #[arg(short = 'd', long = "dir", default_value = ".reqq", global = true)]
    dir: String,

    /// Only print the response body.
    #[arg(short = 'r', long = "raw")]
    raw: bool,

    /// The optional args for the request. Can provide multiple args.
    /// 
    /// Example:
    ///    reqq my-request -a id=1 -a name=foo
    #[arg(
        short = 'a',
        long = "arg",
        action = clap::ArgAction::Append,
        value_parser = clap::builder::ValueParser::new(parse_extra_arg),
    )]
    extra_args: Vec<(String, String)>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists available requests.
    List,

    /// Lists available environments.
    Envs,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let reqq = Reqq::new(ReqqOpts {
        dir: args.dir.as_str(),
        raw: args.raw,
    })?;

    if args.command.is_none() && args.request_name.is_none() {
        eprintln!("Error: 'request_name' is required when no subcommand is specified.");
        std::process::exit(1);
    }

    match &args.command {
        Some(Commands::List) => {
            for req_name in reqq.list_reqs().into_iter() {
                println!("{}", req_name);
            }
        }
        Some(Commands::Envs) => {
            for env_name in reqq.list_envs().into_iter() {
                println!("{}", env_name);
            }
        }
        None => {
            let request_name = args.request_name.as_deref().expect("No request name provided.");
            let extra_args = build_extra_args_map(args.extra_args);
            println!("{}", reqq.execute(request_name, Some(args.env), extra_args)?);
        }
    }
    Ok(())
}

fn build_extra_args_map(cli_extra_args: Vec<(String, String)>) -> HashMap<String, serde_json::Value> {
    let mut extra_args: HashMap<String, serde_json::Value> = HashMap::new();
    for arg in cli_extra_args {
        extra_args.insert(
            arg.0.to_owned(),
            serde_json::to_value(arg.1).unwrap(),
        );
    }
    extra_args
}

fn parse_extra_arg(raw_arg: &str) -> Result<(String, String), std::io::Error> {
    let kv_pair: Vec<&str> = raw_arg.splitn(2, "=").collect();
    if kv_pair.len() < 2 {
        eprintln!("At least one of the args provided is malformed.");
        std::process::exit(1);
    }
    Ok((kv_pair[0].to_owned(), kv_pair[1].to_owned()))
}