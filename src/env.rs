use std::fs;

#[derive(Clone)]
pub struct Env {
    // TODO: Bummer that these only need to be public for `request` tests.
    pub fpath: String,
    pub fstr: Option<String>,
}

type Result<T> = std::result::Result<T, anyhow::Error>;

impl Env {
    pub fn new(fpath: String) -> Self {
        Env { fpath, fstr: None }
    }

    // TODO: Pull this into some kind of Namer trait?
    pub fn name(&self, dir: &str) -> String {
        self.fpath
            .trim_start_matches(dir)
            .trim_start_matches("/envs/")
            .trim_end_matches(".json")
            .into()
    }

    pub fn load(&mut self) -> Result<()> {
        if self.fstr.is_none() {
            let fstr = fs::read_to_string(self.fpath.clone())?;
            self.fstr = Some(fstr);
        }
        Ok(())
    }

    pub fn json(&self) -> Result<serde_json::Value> {
        let v = serde_json::from_str(self.fstr.clone().unwrap().as_str())?;
        Ok(v)
    }
}
