use std::fs;
use regex::Regex;
use handlebars::Handlebars;
use reqwest::{
    Url,
    Method,
    header::{
        HeaderName,
        HeaderValue,
    },
    blocking::{
        Client as ReqwestClient,
        Response,
        RequestBuilder,
    }
};
use anyhow::{anyhow, Result};
use crate::env::Env;

#[derive(Clone)]
pub struct Request {
    fpath: String,
    fstr: Option<String>,
    inner: Option<RequestInner>,
}

#[derive(Clone)]
pub struct RequestInner {
    method: Method,
    url: Url,
    headers: Vec<(HeaderName, HeaderValue)>,
    body: Option<String>,
}

impl Request {
    /// Parses a new request file into a Request struct.
    pub fn new(fpath: String) -> Self {
        Request { fpath, fstr: None, inner: None }
    }

    /// Generates a request name from a config directory and a filename.
    pub fn name(&self, dir: &str) -> String {
        self.fpath
            .trim_start_matches(dir)
            .trim_start_matches("/")
            .trim_end_matches(".reqq")
            .to_owned()
    }

    fn load(&mut self) -> Result<()> {
        if self.fstr.is_none() {
            let fstr = fs::read_to_string(self.fpath.clone())?;
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

    fn parse(&mut self, env: Option<Env>) -> Result<()> {
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
            .ok_or(anyhow!("Failed reading first line."))?
            .splitn(2, " ");

        let method_raw: &[u8] = fline_parts.next()
            .ok_or(anyhow!("Failed reading first line."))?
            .as_bytes();
        let method = Method::from_bytes(method_raw)?;

        let url_raw = fline_parts.next().ok_or(anyhow!("Failed reading first line."))?;
        let url = Url::parse(url_raw)?;

        let header_regex = Regex::new(r"^[A-Za-z0-9-]+:\s*.+$")?;

        let mut headers: Vec<(HeaderName, HeaderValue)> = vec![];
        let mut body: Option<String> = None;

        // Get headers.
        while let Some(line) = lines.next() {
            if !header_regex.is_match(line) {
                // If we have a line that isn't a header, it's the start of the body.
                body = Some(line.to_owned());
                break;
            }

            let mut parts = line.splitn(2, ": ");

            let name = HeaderName::from_bytes(parts.next().unwrap().as_bytes())?;
            let val = HeaderValue::from_bytes(parts.next().unwrap().as_bytes())?;

            headers.push((name, val));
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

    /// Attempt to execute the request with an optional environment configuration file.
    /// This will parse the request first, then send it using reqwest. The resulting
    /// response is formatted and returned as a String.
    pub fn execute(&mut self, env: Option<Env>) -> Result<Response> {
        self.parse(env)?;
        let resp = self.to_reqwest().send()?;
        Ok(resp)
    }

    fn to_reqwest(&self) -> RequestBuilder {
        let client = ReqwestClient::new();

        let mut req = client.request(
            self.inner.clone().unwrap().method,
            self.inner.clone().unwrap().url,
        );

        for (key, val) in self.inner.clone().unwrap().headers {
            req = req.header(key, val);
        }

        if self.inner.clone().unwrap().body.is_some() {
            req = req.body(self.inner.clone().unwrap().body.unwrap());
        }

        req
    }
}

#[test]
fn test_request_name() {
    let dir = ".reqq";
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
    assert!(inner.url.as_str() == "https://example.com/");
    assert!(inner.headers[0].0 == HeaderName::from_bytes("x-example-header".as_bytes()).unwrap());
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
    assert!(inner.url.as_str() == "https://example.com/");
    assert!(inner.headers[0].0 == HeaderName::from_bytes("x-example-header".as_bytes()).unwrap());
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
    assert!(inner.url.as_str() == "https://example.com/");
    assert!(inner.headers[0].0 == HeaderName::from_bytes("x-example-header".as_bytes()).unwrap());
    assert!(inner.headers[0].1 == "lolwat".to_owned());
    assert!(inner.body == Some("\nrequest 5 content".to_owned()));
}
