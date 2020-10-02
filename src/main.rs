use clap::{App, Arg, SubCommand};
use anyhow::Result;
use reqq::{Reqq, ReqqOpts};

fn main() -> Result<()> {
    let matches = App::new("reqq").version("1.0.0")
        .author("Seth Etter <mail@sethetter.com>")
        .about("You know..")
        // TODO: optional --dir to override default of .reqq
        .arg(Arg::with_name("env")
            .short("e")
            .long("env")
            .help("Specifies the environment config file to use")
            .takes_value(true))
        .arg(Arg::with_name("raw")
            .short("r")
            .long("raw")
            .help("Only print the response body."))
        .arg(Arg::with_name("request")
            .help("The name of the request to execute.")
            .index(1))
        .subcommand(SubCommand::with_name("list")
            .about("Lists available requests"))
        .subcommand(SubCommand::with_name("envs")
            .about("Lists available environments"))
        .get_matches();

    let reqq = Reqq::new(ReqqOpts {
        dir: ".reqq",
        raw: matches.is_present("raw"),
    })?;

    if let Some(_) = matches.subcommand_matches("list") {
        for req_name in reqq.list_reqs().into_iter() {
            println!("{}", req_name);
        }
    } else if let Some(_) = matches.subcommand_matches("envs") {
        for env_name in reqq.list_envs().into_iter() {
            println!("{}", env_name);
        }
    } else {
        let req = match matches.value_of("request") {
            Some(r) => r,
            None => {
                eprintln!("Must provide a request");
                std::process::exit(1);
            }
        };
        let env = matches.value_of("env").map(|v| v.to_owned());
        println!("{}", reqq.execute(req, env)?);
    }
    Ok(())
}


