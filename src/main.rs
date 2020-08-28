use clap::{ArgMatches, App, SubCommand};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// First? Start the basic structure of a quicli app.
// How can I support different commands, with a default of some sort?

fn main() {
    println!("Hello, world!");

    let matches = App::new("reqq").version("1.0.0")
        .author("Seth Etter <mail@sethetter.com>")
        .about("You know..")
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
    /// All configured environments.
    envs: Vec<Env>,
}

impl Reqq {
    /// Takes a path to a reqq directory and builds out a Reqq object loaded with
    /// all available request and environment files.
    fn new(dir: String) -> Result<Self> {
        let fpaths = get_all_fpaths(dir)?;
        let env_folder = "envs/";

        // Get request files.
        let reqs: Vec<Request> = fpaths.clone().into_iter().filter_map(|f| {
            if f.starts_with(env_folder) { return None }
            match fs::read_to_string(f.clone()) {
                Ok(fbody) => Request::new(f.clone(), fbody).ok(),
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

        Ok(Reqq { reqs, envs: vec![] })
    }

    fn run(&self, matches: ArgMatches) -> Result<()> {
        if let Some(_) = matches.subcommand_matches("list") {
            // list files in .reqq
            return Ok(());
        }

        Ok(())
    }
}

// TODO: split Request into module?

#[derive(Clone)]
struct Request {
    name: String,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    fn new(name: String, fstr: String) -> Result<Self> {
        let mut lines = fstr.lines().into_iter();

        // Get method and URL.
        let mut fline_parts = lines.next()
            .ok_or("failed parsing first line of request file")?
            .splitn(2, " ");
        let method = fline_parts.next().ok_or("failed to get method")?.to_owned();
        let url = fline_parts.next().ok_or("failed to get url")?.to_owned();

        let header_regex = Regex::new(r"^[A-Za-z0-9-]+: .+$")?;

        let mut headers: Vec<(String, String)> = vec![];
        let mut body: Option<String> = None;

        // Get headers.
        while let Some(line) = lines.next() {
            if !header_regex.is_match(line) {
                // If we have a line that isn't a header, it's the start of the body.
                body = Some(line.to_owned());
                break;
            }

            let mut parts = line.splitn(2, ": ");

            headers.push((
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
            ));
        }

        // Get body.
        if lines.clone().count() > 0 {
            while let Some(line) = lines.next() {
                body = Some(format!("{}\n{}", body.unwrap(), line));
            }
        }

        Ok(Request { name, url, method, headers, body, }.clone())
    }
}

#[test]
fn test_request_file_no_body() {
    let name = "test-request".to_owned();
    let request_file = "GET https://example.com
x-example-header: lolwat".to_owned();

    let req = Request::new(name.clone(), request_file).unwrap();

    assert!(req.name == name.to_owned());
    assert!(req.method.as_str() == "GET");
    assert!(req.headers[0].0 == "x-example-header".to_owned());
    assert!(req.headers[0].1 == "lolwat".to_owned());
    assert!(req.body == None);
}

#[test]
fn test_request_file_with_body() {
    let name = "test-request".to_owned();
    let request_file = "POST https://example.com
x-example-header: lolwat
request body content".to_owned();

    let req = Request::new(name.clone(), request_file).unwrap();

    assert!(req.name == name.to_owned());
    assert!(req.method.as_str() == "POST");
    assert!(req.headers[0].0 == "x-example-header".to_owned());
    assert!(req.headers[0].1 == "lolwat".to_owned());
    assert!(req.body == Some("request body content".to_owned()));
}


struct Env {}

fn get_all_fpaths(dir: String) -> Result<Vec<String>> {
    Ok(
        WalkDir::new(dir).into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(e) => {
                        let path_display = e.path().display().to_string();
                        match path_display.splitn(2, ".reqq/").nth(1) {
                            Some(s) => Some(s.to_owned()),
                            None => None,
                        }
                    },
                    Err(_) => None,
                }
            })
            .collect()
    )
}


