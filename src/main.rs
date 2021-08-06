use ansi_term::Colour::{Green, Red};
use clap::{Arg, App};
use reqwest::header::USER_AGENT;
use serde_json::{self, Value as JsonValue};
use std::error;
use std::fmt::{self, Display};
use std::ops::Add;

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
                write!(f, "{} {:?}", Red.paint("template not found"), vec)
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
    // NOTE: apparently, clap has support to read from a yaml file.
    // will try to use that in the future and avoid all this verbosity

    let matches = App::new("gitignore.rs")
        .version("0.1.0")
        .author("appositum")
        .about("Fetches .gitignore templates from GitHub's API")
        .arg(
            Arg::with_name("list")
                .help("Requests list of all available templates")
                .short("l")
                .long("list"),
        )
        .arg(
            Arg::with_name("templates")
                .help("Comma separated list of templates. e.g.: Rust,Python,C")
                .index(1)
                .required(true)
                .conflicts_with("list"),
        )
        // TODO
        .arg(
            Arg::with_name("file")
                .help("Overwrites .gitignore file with output")
                .short("f"),
        )
        .arg(
            Arg::with_name("append")
                .help("Appends output to .gitignore file")
                .short("a")
                .long("append")
                .conflicts_with("file"),
        )
        .arg(
            Arg::with_name("output")
                .help("Redirects output to a file or stream (default: stdout)")
                .short("o")
                .long("output"),
        )
        .get_matches();

    let api = String::from("https://api.github.com/gitignore/templates");
    let client = reqwest::Client::new();

    // `client.get` consumes the String
    let templates: Vec<String> = get_all_templates(api.clone(), &client).await?;

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
            let usage = String::from(matches.usage())
                .add("\n\nFor more information try ")
                .add(&format!("{}", Green.paint("--help")));

            eprintln!(
                "{} Template(s) not found: {:?}\n\n{}",
                Red.bold().paint("error:"),
                templates_not_found,
                usage
            );

            return Err(GIError::TemplateNotFound(templates_not_found));
        }

        let bodies: Vec<_> = urls
            .map(|url| {
                let client = client.clone();

                // TODO: use `request_body` instead of this block,
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

async fn request_body(url: String, client: &reqwest::Client) -> Result<String, reqwest::Error> {
    Ok(client
        .get(url)
        .header(USER_AGENT, "gitignore.rs")
        .send()
        .await?
        .text()
        .await?)
}

async fn _get_template(name: String, client: &reqwest::Client) -> Result<String, GIError> {
    // NOTE: this looks disgusting. i define this api link in the main function
    // and pass it around as argument to the other request functions.
    // in this `get_template` case, i want to be able to pass only the name of the template
    // and have the api link be implicit, but it's inconsistent
    // with the way the other functions work. gonna figure out a better way
    // to design this. perhaps hardcode the link into the request_body function?
    let url = format!("https://api.github.com/gitignore/templates/{}", name);
    let body = request_body(url, client).await?;

    let data: JsonValue = serde_json::from_str(&body)?;
    let mut result = String::new();

    if let JsonValue::String(name) = &data["name"] {
        if let JsonValue::String(source) = &data["source"] {
            result.push_str(&format!("### {} ###\n{}", name, source));
        }
    }

    Ok(result)
}

async fn get_all_templates(url: String, client: &reqwest::Client) -> Result<Vec<String>, GIError> {
    let body = request_body(url, client).await?;

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
