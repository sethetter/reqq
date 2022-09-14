use reqwest::blocking::Response;
use http::HeaderMap;
use anyhow::Result;

enum ContentType {
    Json,
    Unknown,
}

// TODO: Look at the content-type header and attempt to parse based on content.
pub fn format_response(resp: Response, raw: bool) -> Result<String> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let content_type = get_content_type(headers.clone())?;

    let raw_body: String = resp.text()?;
    let body = format_content_type(content_type, raw_body);

    if raw {
        Ok(body)
    } else {
        let header_lines: Vec<String> = headers.iter().map(|(k, v)| {
            format!("{}: {}", k, v.to_str().unwrap())
        }).collect();

        let mut r = format!("{}\n{}\n", status.as_str(), header_lines.join("\n"));
        r.push_str(body.as_str());
        Ok(r)
    }
}

fn format_content_type(content_type: ContentType, content: String) -> String {
    match content_type {
        ContentType::Json => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(v) => match serde_json::to_string_pretty(&v) {
                    Ok(out) => out,
                    Err(_) => content,
                },
                Err(_) => content,
            }
        },
        ContentType::Unknown => content,
    }
}

fn get_content_type(headers: HeaderMap) -> Result<ContentType> {
    let content_type_header = headers.iter()
        .find(|(k, _)| k.as_str().to_lowercase() == "content-type");

    match content_type_header {
        Some((_, v)) => {
            let v = v.to_str()?.to_lowercase();
            let tokens:Vec<&str> = v.split(';').collect();
            match tokens[0] {
                "application/json" => {
                    Ok(ContentType::Json)
                },
                _ => Ok(ContentType::Unknown),
            }
        },
        None => Ok(ContentType::Unknown),
    }
}
