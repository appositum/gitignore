use ansi_term::Colour::Red;
use clap::{App, load_yaml};
use req::header::USER_AGENT;
use serde_json::{self, Value as JsonValue};
use std::error;
use std::fmt::{self, Display};
use reqwest as req;

#[derive(Debug)]
enum GIError {
    Json(serde_json::Error),
    Request(req::Error),
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

impl From<req::Error> for GIError {
    fn from(err: req::Error) -> GIError {
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

    let client = req::Client::new();

    // `client.get` consumes the String
    let all_templates: Vec<String> = get_all_templates(&client).await?;

    if matches.is_present("list") {
        pretty_print(all_templates);

        return Ok(());
    }

    if let Some(ts) = matches.value_of("templates") {
        // this needs to be a vector so we can iterate through the values as references,
        // that way, the for loop wont consume it. also, we're gonna pass this to
        // `get_bodies`, which takes a vector anyway.
        let templates_input: Vec<String> = ts.split(',').map(String::from).collect();

        let mut templates_not_found: Vec<String> = Vec::new();

        for t in &templates_input {
            if !all_templates.contains(t) {
                templates_not_found.push(t.clone());
            }
        }

        if !templates_not_found.is_empty() {
            // NOTE: printing the error looks nicer than
            // having the debug structure returned from `main`,
            // i might rewrite the main function later,
            // and add a library to the project
            //
            // eprintln!("{}", GIError::TemplateNotFound(templates_not_found.clone()));
            return Err(GIError::TemplateNotFound(templates_not_found));
        }

        let result_templates: Vec<JsonValue> =
            get_bodies(&client, templates_input).await?;

        for t in result_templates {
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
    client: &req::Client,
    template_name: Option<String>,
) -> Result<req::Response, req::Error> {
    let api = String::from("https://api.github.com/gitignore/templates");

    let url = match template_name {
        None => api,
        Some(template) => format!("{}/{}", api, template),
    };

    Ok(client
        .get(url)
        .header(USER_AGENT, "gitignore.rs")
        .send()
        .await?)
}

// NOTE: i'll probably copy-paste this bit of code into `get_bodies`
async fn _get_template(
    client: &req::Client,
    template_name: String,
) -> Result<String, GIError> {
    let body = request_api(client, Some(template_name))
        .await?
        .text()
        .await?;

    let data: JsonValue = serde_json::from_str(&body)?;
    let mut result = String::new();

    if let JsonValue::String(name) = &data["name"] {
        if let JsonValue::String(source) = &data["source"] {
            result.push_str(&format!("### {} ###\n{}", name, source));
        }
    }

    Ok(result)
}

async fn get_all_templates(client: &req::Client) -> Result<Vec<String>, GIError> {
    let body = request_api(client, None).await?.text().await?;

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

async fn get_bodies(
    client: &req::Client,
    template_list: Vec<String>,
) -> Result<Vec<JsonValue>, GIError> {
    let mut templates: Vec<JsonValue> = Vec::new();

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
                let template = serde_json::from_str(&b)?;
                templates.push(template);
            }
        }
    }

    Ok(templates)
}

// NOTE: i wonder if there's a prettier way to write this function.
// the amount of `.clone()` bothers me
fn pretty_print(list: Vec<String>) {
    // [1, 2, 3, 4, 5, 6, 7] -> [[1, 2, 3], [4, 5, 6], [7]]
    let chunks = list.chunks(3);

    // get length of the biggest string from subgroup
    let max1 = chunks
        .clone()
        .map(|subgroup| subgroup[0].len())
        .max()
        .unwrap();

    let max2 = chunks
        .clone()
        .map(|subgroup| {
            if subgroup.len() < 2 {
                subgroup[0].len()
            } else {
                subgroup[1].len()
            }
        })
        .max()
        .unwrap();

    // turn into a Vec<(&str, &str, &str)>
    chunks
        .map(|subgroup| {
            if subgroup.len() == 1 {
                (subgroup[0].clone(), String::new(), String::new())
            } else if subgroup.len() == 2 {
                (subgroup[0].clone(), subgroup[1].clone(), String::new())
            } else {
                (
                    subgroup[0].clone(),
                    subgroup[1].clone(),
                    subgroup[2].clone(),
                )
            }
        })
        .for_each(|(x, y, z)| {
            println!("{:<w1$}\t{:<w2$}\t{}", x, y, z, w1 = max1, w2 = max2);
        })
}
