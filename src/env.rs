#[derive(Clone)]
pub struct Env {
    fpath: String,
}

impl Env {
    pub fn new(fpath: String) -> Self {
        Env { fpath }
    }

    // TODO: Pull this into some kind of Namer trait?
    pub fn name(&self, dir: String) -> String {
        self.fpath
            .trim_start_matches(dir.as_str())
            .trim_start_matches("/envs/")
            .trim_end_matches(".json")
            .to_owned()
    }

}
