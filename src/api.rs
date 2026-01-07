use crate::error::GIError;

use reqwest as req;
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

pub async fn get_template_list(client: &req::Client) -> Result<Vec<String>, GIError> {
    let body = request_api(client, None).await?.text().await?;
    let data: TemplateList = to_json(&body)?;

    Ok(data.0)
}

pub async fn get_templates(
    client: &req::Client,
    template_list: Vec<String>,
) -> Result<Vec<Template>, GIError> {
    let mut templates: Vec<Template> = Vec::new();

    let bodies: Vec<_> = template_list
        .into_iter()
        .map(|t| {
            let client = client.clone();

            tokio::spawn(async move { request_api(&client, Some(t)).await?.text().await })
        })
        .collect();

    for body in bodies {
        match body.await {
            Err(e) => return Err(GIError::TaskJoin(e)),
            Ok(Err(e)) => return Err(GIError::Request(e)),
            Ok(Ok(b)) => {
                let mut template: Template = to_json(&b)?;

                // we're trimming this because the number of newlines
                // at the end of the response data is inconsistent.
                // the C template ends with a single newline,
                // but the Lua template ends with two newlines.
                template.source = template.source.trim().to_string();
                templates.push(template);
            },
        }
    }

    Ok(templates)
}

async fn request_api(
    client: &req::Client,
    template_name: Option<String>,
) -> Result<req::Response, req::Error> {
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
    .send()
    .await?)
}
