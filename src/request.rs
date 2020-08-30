use std::fs;
use regex::Regex;
use thiserror::Error;
use handlebars::Handlebars;
use reqwest::{Url, Method, header, blocking as r};
use crate::env::{Env, EnvError};

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to read request file")]
    ReadError,
    #[error(transparent)]
    CompileError(#[from] handlebars::TemplateRenderError),
    #[error("Failed to parse request file")]
    ParseError,
    #[error(transparent)]
    EnvError(#[from] EnvError),
    #[error(transparent)]
    MethodError(#[from] http::method::InvalidMethod),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, RequestError>;

#[derive(Clone)]
pub struct Request {
    fpath: String,
    fstr: Option<String>,
    inner: Option<RequestInner>,
}

// TODO: Better name for this type.
#[derive(Clone)]
pub struct RequestInner {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    /// Parses a new request file into a Request struct.
    pub fn new(fpath: String) -> Self {
        Request { fpath, fstr: None, inner: None }
    }

    /// Generates a request name from a config directory and a filename.
    pub fn name(&self, dir: String) -> String {
        self.fpath
            .trim_start_matches(dir.as_str())
            .trim_start_matches("/")
            .trim_end_matches(".reqq")
            .to_owned()
    }

    fn load(&mut self) -> Result<()> {
        if self.fstr.is_none() {
            let fstr = fs::read_to_string(self.fpath.clone())
                .map_err(|_| RequestError::ReadError)?;
            self.fstr = Some(fstr);
        }
        Ok(())
    }

    fn apply_env(&mut self, mut env: Env) -> Result<()> {
        env.load()?;

        let reg = Handlebars::new();
        let result = reg.render_template(
            self.fstr.clone().unwrap().as_str(),
            &env.json()?,
        )?;

        self.fstr = Some(result);

        Ok(())
    }

    pub fn parse(&mut self, env: Option<Env>) -> Result<()> {
        // Make sure we have the file content loaded.
        if self.fstr == None { self.load()?; }

        // If an env is provided, parse the request file with it applied.
        if let Some(env) = env {
            self.apply_env(env)?;
        }

        // Parse the request file.
        let fstr = self.fstr.clone().unwrap();
        let mut lines = fstr.lines().into_iter();

        // Get method and URL.
        let mut fline_parts = lines.next()
            .ok_or("failed parsing first line of request file")
            .map_err(|_| RequestError::ParseError)?
            .splitn(2, " ");
        let method = fline_parts.next()
            .ok_or("failed to get method")
            .map_err(|_| RequestError::ParseError)?
            .to_owned();
        let url = fline_parts.next()
            .ok_or("failed to get url")
            .map_err(|_| RequestError::ParseError)?
            .to_owned();

        let header_regex = Regex::new(r"^[A-Za-z0-9-]+: .+$")
            .map_err(|_| RequestError::ParseError)?;

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

        self.inner = Some(RequestInner{ url, method, headers, body });

        Ok(())
    }

    pub fn execute(&mut self, env: Option<Env>) -> Result<String> {
        self.parse(env)?;

        // TODO: actually send the request via reqwest.
        let client = r::Client::new();
        let mut req = client.request(
            Method::from_bytes(self.inner.clone().unwrap().method.as_bytes())?,
            Url::parse(self.inner.clone().unwrap().url.as_str())?,
        );

        for (key, val) in self.inner.clone().unwrap().headers {
            let name = header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|_| RequestError::ParseError)?;
            req = req.header(name, val);
        }

        if self.inner.clone().unwrap().body.is_some() {
            req = req.body(self.inner.clone().unwrap().body.unwrap());
        }

        let resp = req.send()?;

        let status = resp.status();
        let header_lines: Vec<String> = resp.headers().iter().map(|(k, v)| {
            format!("{}: {}", k, v.to_str().unwrap())
        }).collect();
        let body = resp.text()?;

        Ok(format!("{}\n{}\n{}", status.as_str(), header_lines.join("\n"), body))
    }
}

#[test]
fn test_request_name() {
    let dir = ".reqq".to_owned();
    let fpath = ".reqq/nested/example-request.reqq".to_owned();

    let req = Request::new(fpath);
    assert!(req.name(dir) == "nested/example-request".to_owned());
}

#[test]
fn test_request_file_no_body() {
    let fpath = ".reqq/nested/exammple-request.reqq".to_owned();
    let fstr = "GET https://example.com
x-example-header: lolwat".to_owned();

    let mut req = Request::new(fpath);
    req.fstr = Some(fstr);

    req.parse(None).expect("Failed to parse request.");
    let inner = req.clone().inner.unwrap();

    assert!(inner.method.as_str() == "GET");
    assert!(inner.url.as_str() == "https://example.com");
    assert!(inner.headers[0].0 == "x-example-header".to_owned());
    assert!(inner.headers[0].1 == "lolwat".to_owned());
    assert!(inner.body == None);
}

#[test]
fn test_request_file_with_body() {
    let fpath = ".reqq/nested/exammple-request.reqq".to_owned();
    let fstr = "POST https://example.com
x-example-header: lolwat

request body content".to_owned();

    let mut req = Request::new(fpath);
    req.fstr = Some(fstr);

    req.parse(None).expect("Failed to parse request.");
    let inner = req.clone().inner.unwrap();

    assert!(inner.method.as_str() == "POST");
    assert!(inner.url.as_str() == "https://example.com");
    assert!(inner.headers[0].0 == "x-example-header".to_owned());
    assert!(inner.headers[0].1 == "lolwat".to_owned());
    assert!(inner.body == Some("\nrequest body content".to_owned()));
}

#[test]
fn test_request_with_env() {
    let fpath = ".reqq/nested/exammple-request.reqq".to_owned();
    let fstr = "POST https://example.com
x-example-header: {{ headerVal }}

request {{ shwat }} content".to_owned();

    let env_str = "{\"headerVal\": \"lolwat\", \"shwat\": 5 }".to_owned();
    let env = Env { fpath: "".to_owned(), fstr: Some(env_str) };

    let mut req = Request::new(fpath);
    req.fstr = Some(fstr);

    req.parse(Some(env)).expect("Failed to parse request.");
    let inner = req.clone().inner.unwrap();

    assert!(inner.method.as_str() == "POST");
    assert!(inner.url.as_str() == "https://example.com");
    assert!(inner.headers[0].0 == "x-example-header".to_owned());
    assert!(inner.headers[0].1 == "lolwat".to_owned());
    assert!(inner.body == Some("\nrequest 5 content".to_owned()));
}
