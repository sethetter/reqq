use clap::{ArgMatches, App, SubCommand};
use std::fs;
use walkdir::WalkDir;
use reqq::request::Request;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// First? Start the basic structure of a quicli app.
// How can I support different commands, with a default of some sort?

fn main() {
    println!("Hello, world!");

    let matches = App::new("reqq").version("1.0.0")
        .author("Seth Etter <mail@sethetter.com>")
        .about("You know..")

        // TODO: optional --dir option to override default of .reqq

        // .arg(Arg::with_name("env")
        //     .short("e")
        //     .long("env")
        //     .value_name("ENV")
        //     .help("Specifies the environment config file to use")
        //     .takes_value(true))

        .subcommand(SubCommand::with_name("list")
            .about("Lists available requests"))
        .get_matches();

    let app = Reqq::new(".reqq".to_owned()).unwrap(); // maybe make this better?

    match app.run(matches) {
        Err(_) => println!("failed!"),
        Ok(_) => {},
    };
}

/// The top level app object which loads all available requests and environments
/// so that various user actions can be performed with them.
struct Reqq {
    /// All available request files.
    reqs: Vec<Request>,
    // /// All configured environments.
    // envs: Vec<Env>,
}

impl Reqq {
    /// Takes a path to a reqq directory and builds out a Reqq object loaded with
    /// all available request and environment files.
    fn new(dir: String) -> Result<Self> {
        let fpaths = get_all_fpaths(dir.clone())?;
        let env_folder = "envs/";

        // Get request files.
        let reqs: Vec<Request> = fpaths.clone().into_iter().filter_map(|f| {
            if f.starts_with(env_folder) { return None }
            match fs::read_to_string(f.clone()) {
                Ok(fbody) => Request::new(Request::name(dir.clone(), f.clone()), fbody).ok(),
                Err(_) => None,
            }
        }).collect();

        // Get environments.
        // let envs: Vec<Request> = fpaths.clone().into_iter().filter_map(|f| {
        //     if !f.starts_with(env_folder) { return None }
        //     match fs::read_to_string(f.clone()) {
        //         Ok(fbody) => Env::new(f.clone(), fbody).ok(),
        //         Err(_) => None,
        //     }
        // }).collect();

        Ok(Reqq { reqs })
    }

    fn run(&self, matches: ArgMatches) -> Result<()> {
        if let Some(_) = matches.subcommand_matches("list") {
            for r in self.reqs.clone() {
                println!("{}", r.name);
            }
            return Ok(());
        }

        Ok(())
    }
}

// TODO: This is gross.
fn get_all_fpaths(dir: String) -> Result<Vec<String>> {
    Ok(
        WalkDir::new(dir.clone()).into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(e) => {
                        let path_display = e.path().display().to_string();
                        if path_display.as_str().trim_start_matches(&dir) == "" {
                            return None;
                        }
                        Some(path_display)
                    },
                    Err(_) => None,
                }
            })
            .collect()
    )
}
