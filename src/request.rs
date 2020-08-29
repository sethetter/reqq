use std::fmt;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Request {
    pub name: String,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    /// Parses a new request file into a Request struct.
    pub fn new(name: String, fstr: String) -> Result<Self> {
        let mut lines = fstr.lines().into_iter();

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

        Ok(Request { name, url, method, headers, body, }.clone())
    }

    /// Generates a request name from a config directory and a filename.
    pub fn name(dir: String, fname: String) -> String {
        fname
            .trim_start_matches(dir.as_str())
            .trim_start_matches("/")
            .trim_end_matches(".reqq")
            .to_owned()
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
