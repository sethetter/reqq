use clap::ArgMatches;
use std::fs;
use walkdir::WalkDir;
use crate::request::Request;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ArgsError,
    RequestError,
}

/// The top level app object which loads all available requests and environments
/// so that various user actions can be performed with them.
pub struct Reqq {
    /// All available request files.
    reqs: Vec<Request>,
    // /// All configured environments.
    // envs: Vec<Env>,
}

impl Reqq {
    /// Takes a path to a reqq directory and builds out a Reqq object loaded with
    /// all available request and environment files.
    pub fn new(dir: String) -> Result<Self> {
        let fpaths = get_all_fpaths(dir.clone())?;
        let env_folder = "envs/";

        // Get request files.
        let reqs: Vec<Request> = fpaths.clone().into_iter().filter_map(|f| {
            if f.starts_with(env_folder) { return None }
            match fs::read_to_string(f.clone()) {
                Ok(fstr) => Some(Request::new(dir.clone(), f.to_string(), fstr)),
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

    /// Accepts parsed `clap::ArgMatches` and performs the requested action.
    pub fn run(&self, matches: ArgMatches) -> Result<()> {
        if let Some(_) = matches.subcommand_matches("list") {
            for r in self.reqs.clone() {
                println!("{}", r.name());
            }
            return Ok(());
        }

        let req = matches.value_of("REQUEST").ok_or(Error::ArgsError)?;
        self.execute(req.to_owned())?;

        Ok(())
    }

    fn execute(&self, _req: String) -> Result<()> {
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
