use walkdir::WalkDir;
use thiserror::Error;
use crate::{
    request::Request,
    env::Env,
};

type Result<T> = std::result::Result<T, ReqqError>;

#[derive(Debug, Error)]
pub enum ReqqError {
    #[error("Request not found: {0}")]
    RequestNotFound(String),
    // #[error("")]
    // FailedToParseRequest { },
}

/// The top level app object which loads all available requests and environments
/// so that various user actions can be performed with them.
pub struct Reqq {
    dir: String,
    reqs: Vec<Request>,
    envs: Vec<Env>,
}

impl Reqq {
    // TODO: Decouple the IO portions of this somehow?
    /// Takes a path to a reqq directory and builds out a Reqq object loaded with
    /// all available request and environment files.
    pub fn new(dir: String) -> Result<Self> {
        let fpaths = get_all_fpaths(dir.clone());
        let env_folder = format!("{}/{}", dir, "envs");

        // Get request files.
        let reqs: Vec<Request> = fpaths.clone().into_iter().filter_map(|f| {
            if f.starts_with(env_folder.as_str()) { return None }
            Some(Request::new(f.to_string()))
        }).collect();

        // Get environments.
        let envs: Vec<Env> = fpaths.clone().into_iter().filter_map(|f| {
            if !f.starts_with(env_folder.as_str()) || f == env_folder {
                return None
            }
            Some(Env::new(f.to_string()))
        }).collect();

        Ok(Reqq { dir, reqs, envs })
    }

    /// Provide a list of all available request names.
    pub fn list_reqs(&self) -> Vec<String> {
        self.reqs.clone().into_iter().map(|r| r.name(self.dir.clone())).collect()
    }

    /// Provide a list of all available environment names.
    pub fn list_envs(&self) -> Vec<String> {
        self.envs.clone().into_iter().map(|r| r.name(self.dir.clone())).collect()
    }

    // /// Executes a specified request, optionally with an environment.
    // fn execute(&self, _req: String, _env: Option<String>) -> Result<()> {
    //     Ok(())
    // }
}

// TODO: This is gross.
fn get_all_fpaths(dir: String) -> Vec<String> {
    WalkDir::new(dir.clone()).into_iter().filter_map(|entry| {
        match entry {
            Ok(e) => {
                let path_display = e.path().display().to_string();
                match path_display.as_str().trim_start_matches(&dir) {
                    "" => None,
                    _ => Some(path_display),
                }
            },
            Err(_) => None,
        }
    })
    .collect()
}
