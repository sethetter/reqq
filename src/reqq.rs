use walkdir::WalkDir;
use crate::{
    request::Request,
    env::Env,
    format::format_response,
};
use anyhow::{anyhow, Result};

/// The top level app object which loads all available requests and environments
/// so that various user actions can be performed with them.
pub struct Reqq<'a> {
    dir: &'a str,
    reqs: Vec<Request>,
    envs: Vec<Env>,
    raw: bool,
}

pub struct ReqqOpts<'a> {
    pub dir: &'a str,
    pub raw: bool,
}

impl <'a>Reqq<'a> {
    // TODO: Decouple the IO portions of this somehow?
    /// Takes a path to a reqq directory and builds out a Reqq object loaded with
    /// all available request and environment files.
    pub fn new(opts: ReqqOpts<'a>) -> Result<Self> {
        let dir = opts.dir;

        let fpaths = get_all_fpaths(dir);
        let env_folder = format!("{}/{}", dir, "envs");

        // Get request files.
        let reqs: Vec<Request> = fpaths.clone().into_iter()
            .filter_map(|f| {
                if f.starts_with(env_folder.as_str()) {
                    return None
                }
                Some(Request::new(f))
            }).collect();

        // Get environments.
        let envs: Vec<Env> = fpaths.into_iter()
            .filter_map(|f| {
                if !f.starts_with(env_folder.as_str()) {
                    return None
                }
                Some(Env::new(f))
            }).collect();

        Ok(Reqq { dir, reqs, envs, raw: opts.raw })
    }

    /// Provide a list of all available request names.
    pub fn list_reqs(&self) -> Vec<String> {
        self.reqs.clone().into_iter()
            .map(|r| r.name(self.dir)).collect()
    }

    /// Provide a list of all available environment names.
    pub fn list_envs(&self) -> Vec<String> {
        self.envs.clone().into_iter()
            .map(|e| e.name(self.dir)).collect()
    }

    /// Executes a request specified by name, optionally with an environment.
    pub fn execute(
        &self,
        req_name: &str,
        env_name: Option<String>,
    ) -> Result<String> {
        let mut req = self.get_req(req_name)?;
        let maybe_env = env_name.map(|n| self.get_env(n)).unwrap();
        let resp = req.execute(maybe_env)?;
        let result = format_response(resp, self.raw)?;
        Ok(result)
    }

    fn get_req(&self, name: &str) -> Result<Request> {
        self.reqs.clone().into_iter()
            .find(|r| r.name(self.dir) == name)
            .ok_or_else(|| anyhow!("Request not found."))
    }

    fn get_env(&self, name: String) -> Option<Env> {
        self.envs.clone().into_iter()
            .find(|e| e.name(self.dir) == name)
    }

}

// TODO: This is gross.
fn get_all_fpaths(dir: &str) -> Vec<String> {
    WalkDir::new(dir).into_iter().filter_map(|entry| {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() {
                    return None;
                }

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
