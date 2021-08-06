use ansi_term::Colour::Red;
use clap::{App, load_yaml};
use reqwest::header::USER_AGENT;
use serde_json::{self, Value as JsonValue};
use std::error;
use std::fmt::{self, Display};

#[derive(Debug)]
enum GIError {
    Json(serde_json::Error),
    Request(reqwest::Error),
    TaskJoin(tokio::task::JoinError),
    TemplateNotFound(Vec<String>),
}

impl Display for GIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GIError::Json(e) => {
                write!(f, "{} {}", Red.paint("json parse error:"), e)
            }
            GIError::Request(e) => {
                write!(f, "{} {}", Red.paint("request error:"), e)
            }
            GIError::TaskJoin(e) => {
                write!(f, "{} {}", Red.paint("tokio join error:"), e)
            }
            GIError::TemplateNotFound(vec) => {
                write!(f, "{} template not found {:?}", Red.paint("error:"), vec)
            }
        }
    }
}

impl error::Error for GIError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            GIError::Json(e) => Some(e),
            GIError::Request(e) => Some(e),
            GIError::TaskJoin(e) => Some(e),
            GIError::TemplateNotFound(_vec) => None,
        }
    }
}

impl From<serde_json::Error> for GIError {
    fn from(err: serde_json::Error) -> GIError {
        GIError::Json(err)
    }
}

impl From<reqwest::Error> for GIError {
    fn from(err: reqwest::Error) -> GIError {
        GIError::Request(err)
    }
}

impl From<tokio::task::JoinError> for GIError {
    fn from(err: tokio::task::JoinError) -> GIError {
        GIError::TaskJoin(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), GIError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let api = String::from("https://api.github.com/gitignore/templates");
    let client = reqwest::Client::new();

    // `client.get` consumes the String
    let templates: Vec<String> = get_all_templates(&client).await?;

    if matches.is_present("list") {
        for t in templates {
            println!("{}", t);
        }

        return Ok(());
    }

    if let Some(ts) = matches.value_of("templates") {
        let templates_input = ts.split(',').map(String::from);
        let urls = templates_input.clone().map(|t| format!("{}/{}", api, t));

        let mut templates_not_found: Vec<String> = Vec::new();

        for t in templates_input {
            if !templates.contains(&t) {
                templates_not_found.push(t);
            }
        }

        if !templates_not_found.is_empty() {
            // NOTE: printing the error looks nicer than
            // having the debug structure returned from `main`,
            // i might rewrite the main function later,
            // possibly add a library to the project
            //
            // eprintln!("{}", GIError::TemplateNotFound(templates_not_found.clone()));
            return Err(GIError::TemplateNotFound(templates_not_found));
        }

        let bodies: Vec<_> = urls
            .map(|url| {
                let client = client.clone();

                // TODO: use `request_api` instead of this block,
                // but types are mistmatching. getting rid of this repetition,
                // we can drop the `urls` variable and use `get_template` instead
                tokio::spawn(async move {
                    client
                        .get(url)
                        .header(USER_AGENT, "gitignore.rs")
                        .send()
                        .await?
                        .text()
                        .await
                })
            })
            .collect();

        let mut templates: Vec<JsonValue> = Vec::new();

        for body in bodies {
            match body.await {
                Err(e) => {
                    return Err(GIError::TaskJoin(e));
                }
                Ok(Err(e)) => {
                    return Err(GIError::Request(e));
                }
                Ok(Ok(b)) => {
                    let template = serde_json::from_str(&b)?;
                    templates.push(template);
                }
            }
        }

        for t in templates {
            if let JsonValue::String(name) = &t["name"] {
                if let JsonValue::String(source) = &t["source"] {
                    // git ignore copypasta output
                    println!("### {} ###\n{}", name, source);
                }
            }
        }
    }

    Ok(())
}

async fn request_api(
    client: &reqwest::Client,
    template_name: Option<String>,
) -> Result<String, reqwest::Error> {
    let api = String::from("https://api.github.com/gitignore/templates");

    let url = match template_name {
        None => api,
        Some(template) => format!("{}/{}", api, template),
    };

    Ok(client
        .get(url)
        .header(USER_AGENT, "gitignore.rs")
        .send()
        .await?
        .text()
        .await?)
}

async fn _get_template(client: &reqwest::Client, template_name: String) -> Result<String, GIError> {
    let body = request_api(client, Some(template_name)).await?;

    let data: JsonValue = serde_json::from_str(&body)?;
    let mut result = String::new();

    if let JsonValue::String(name) = &data["name"] {
        if let JsonValue::String(source) = &data["source"] {
            result.push_str(&format!("### {} ###\n{}", name, source));
        }
    }

    Ok(result)
}

async fn get_all_templates(client: &reqwest::Client) -> Result<Vec<String>, GIError> {
    let body = request_api(client, None).await?;

    let mut result: Vec<String> = Vec::new();

    if let JsonValue::Array(ts) = serde_json::from_str(&body)? {
        for t in ts {
            if let JsonValue::String(s) = t {
                result.push(s);
            }
        }
    }

    Ok(result)
}
