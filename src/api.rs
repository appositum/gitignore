use crate::cache;
use crate::error::GIError;

use reqwest::Error as RequestError;
use reqwest::blocking as req;
use reqwest::header::{
    ACCEPT,
    HeaderMap,
    USER_AGENT,
};
use serde::Deserialize;
use serde_json::from_str as to_json;

#[derive(Deserialize, Debug)]
pub struct Template {
    pub name: String,
    pub source: String,
}

// only using this to deserialize the json array we get
// from requesting the first API endpoint with all templates
#[derive(Deserialize, Debug)]
struct TemplateList(Vec<String>);

fn request_api(
    client: &req::Client,
    template_name: Option<&String>,
) -> Result<req::Response, RequestError> {
    let api = String::from("https://api.github.com/gitignore/templates");

    let url = match template_name {
        None => api,
        Some(template) => format!("{}/{}", api, template),
    };

    let mut hs = HeaderMap::new();
    hs.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
    hs.insert(
        USER_AGENT,
        format!("gitignore.rs {}", env!("CARGO_PKG_VERSION"))
            .parse()
            .unwrap(),
    );
    hs.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

    let client_request = client.get(url).headers(hs);

    Ok(match std::env::var("GITHUB_TOKEN") {
        Err(_) => client_request,
        Ok(token) => client_request.header("Authorization", format!("Bearer {}", token)),
    }
    .send()?)
}

pub fn get_template_list(client: &req::Client) -> Result<Vec<String>, GIError> {
    let body = request_api(client, None)?.text()?;
    let data: TemplateList = to_json(&body)?;

    Ok(data.0)
}

pub fn get_template_contents(
    client: &req::Client,
    template_list: Vec<String>,
) -> Result<Vec<Template>, GIError> {
    let mut templates: Vec<Template> = Vec::new();

    for t in template_list {
        let content: String = match cache::get_cached_template(&t) {
            Some(cached) => cached,
            None => {
                let body = request_api(client, Some(&t))?.text()?;
                cache::cache_template(&t, &body);
                body
            },
        };

        let mut template: Template = to_json(&content)?;

        // we're trimming this because the number of newlines
        // at the end of the response data is inconsistent.
        // the C template ends with a single newline,
        // but the Lua template ends with two newlines.
        template.source = template.source.trim().to_string();
        templates.push(template);
    }

    Ok(templates)
}
