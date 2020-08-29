use std::fmt;
use regex::Regex;
use crate::env::Env;

// TODO: Use thiserror?
#[derive(Debug)]
enum Error {
    ParseError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError => write!(f, "Failed to parse the reqq file."),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Request {
    pub fpath: String,
    pub fstr: String, // TODO: More general type?
    pub inner: Option<RequestInner>,
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
    pub fn new(fpath: String, fstr: String) -> Self {
        Request { fpath, fstr, inner: None }
    }

    /// Generates a request name from a config directory and a filename.
    pub fn name(&self, dir: String) -> String {
        self.fpath
            .trim_start_matches(dir.as_str())
            .trim_start_matches("/")
            .trim_end_matches(".reqq")
            .to_owned()
    }

    // TODO: Refactor this, use nom maybe?
    // TODO: Compile with env support!
    fn compile(&mut self, _env: Option<Env>) -> Result<()> {
        let mut lines = self.fstr.lines().into_iter();

        // Get method and URL.
        let mut fline_parts = lines.next()
            .ok_or("failed parsing first line of request file")
            .map_err(|_| Error::ParseError)?
            .splitn(2, " ");
        let method = fline_parts.next()
            .ok_or("failed to get method")
            .map_err(|_| Error::ParseError)?
            .to_owned();
        let url = fline_parts.next()
            .ok_or("failed to get url")
            .map_err(|_| Error::ParseError)?
            .to_owned();

        let header_regex = Regex::new(r"^[A-Za-z0-9-]+: .+$")
            .map_err(|_| Error::ParseError)?;

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
}

#[test]
fn test_request_name() {
    let dir = ".reqq".to_owned();
    let fpath = ".reqq/nested/example-request.reqq".to_owned();
    let fstr = "what".to_owned();

    let req = Request::new(fpath, fstr);
    assert!(req.name(dir) == "nested/example-request".to_owned());
}

#[test]
fn test_request_file_no_body() {
    let fpath = ".reqq/nested/exammple-request.reqq".to_owned();
    let fstr = "GET https://example.com
x-example-header: lolwat".to_owned();

    let mut req = Request::new(fpath, fstr);
    req.compile(None).expect("Failed to compile.");
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

    let mut req = Request::new(fpath, fstr);
    req.compile(None).expect("Failed to compile.");
    let inner = req.clone().inner.unwrap();

    assert!(inner.method.as_str() == "POST");
    assert!(inner.url.as_str() == "https://example.com");
    assert!(inner.headers[0].0 == "x-example-header".to_owned());
    assert!(inner.headers[0].1 == "lolwat".to_owned());
    assert!(inner.body == Some("\nrequest body content".to_owned()));
}
