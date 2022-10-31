use anyhow::Result;
use clap::{App, Arg, ArgMatches, SubCommand};
use reqq::{Reqq, ReqqOpts};
use std::collections::HashMap;

fn main() -> Result<()> {
    let matches = App::new("reqq")
        .version("0.3.0")
        .author("Seth Etter <mail@sethetter.com>")
        .about("You know..")
        // TODO: optional --dir to override default of .reqq
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("env")
                .help("Specifies the environment config file to use")
                .default_value("default")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("dir")
                .help("Configuration directory to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("raw")
                .short("r")
                .long("raw")
                .help("Only print the response body."),
        )
        .arg(
            Arg::with_name("arg")
                .short("a")
                .long("arg")
                .multiple(true)
                .number_of_values(1)
                .help("The optional args for the request.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("request")
                .help("The name of the request to execute.")
                .index(1),
        )
        .subcommand(SubCommand::with_name("list").about("Lists available requests"))
        .subcommand(SubCommand::with_name("envs").about("Lists available environments"))
        .get_matches();

    let reqq = Reqq::new(ReqqOpts {
        dir: matches.value_of("dir").unwrap_or(".reqq"),
        raw: matches.is_present("raw"),
    })?;

    match parse_command(matches.clone()) {
        Cmd::List => {
            for req_name in reqq.list_reqs().into_iter() {
                println!("{}", req_name);
            }
        }
        Cmd::Envs => {
            for env_name in reqq.list_envs().into_iter() {
                println!("{}", env_name);
            }
        }
        Cmd::Request => {
            let req = match matches.value_of("request") {
                Some(r) => r,
                None => {
                    eprintln!("Must provide a request");
                    std::process::exit(1);
                }
            };
            let env = matches.value_of("env").map(|v| v.to_owned());
            let extra_args: HashMap<String, serde_json::Value> = match matches.values_of("arg") {
                Some(cli_extra_args) => extract_extra_args(cli_extra_args),
                None => HashMap::new(),
            };
            println!("{}", reqq.execute(req, env, extra_args)?);
        }
    }
    Ok(())
}

enum Cmd {
    List,
    Envs,
    Request,
}

fn extract_extra_args(cli_extra_args: clap::Values) -> HashMap<String, serde_json::Value> {
    let mut extra_args: HashMap<String, serde_json::Value> = HashMap::new();
    for arg in cli_extra_args {
        let kv_pair: Vec<&str> = arg.splitn(2, "=").collect();
        if kv_pair.len() < 2 {
            eprintln!("At least one of the args provided is malformed.");
            std::process::exit(1);
        }
        extra_args.insert(
            kv_pair[0].to_owned(),
            serde_json::to_value(kv_pair[1]).unwrap(),
        );
    }
    extra_args
}

fn parse_command(matches: ArgMatches) -> Cmd {
    if matches.subcommand_matches("list").is_some() {
        Cmd::List
    } else if matches.subcommand_matches("envs").is_some() {
        Cmd::Envs
    } else {
        Cmd::Request
    }
}
