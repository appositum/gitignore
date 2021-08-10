use ansi_term::Colour::Red;
use clap::{App, load_yaml};
use reqwest as req;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::{self, from_str as to_json};
use std::error;
use std::fmt::{self, Display};

#[derive(Debug)]
enum GIError {
    Json(serde_json::Error),
    Request(req::Error),
    TaskJoin(tokio::task::JoinError),
    TemplateNotFound(Vec<String>),
}

#[derive(Deserialize, Debug)]
struct Template {
    name: String,
    source: String,
}

// only using this to deserialize the json array we get
// from requesting the first API endpoint with all templates
#[derive(Deserialize, Debug)]
struct TemplateList(Vec<String>);

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

    let all_templates: Vec<String> = get_template_list(&client).await?;

    if matches.is_present("list") {
        pretty_print(all_templates);

        return Ok(());
    }

    if let Some(ts) = matches.value_of("templates") {
        // this needs to be a vector so we can iterate through the values as references,
        // that way, the for loop wont consume it. also, we're gonna pass this to
        // `get_templates`, which takes a vector anyway.
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

        get_templates(&client, templates_input)
            .await?
            .into_iter()
            .for_each(|t| {
                println!("### {} ###\n{}", t.name, t.source);
            });
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

async fn get_template_list(client: &req::Client) -> Result<Vec<String>, GIError> {
    let body = request_api(client, None).await?.text().await?;
    let data: TemplateList = to_json(&body)?;

    Ok(data.0)
}

async fn get_templates(
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
                let template: Template = to_json(&b)?;
                templates.push(template);
            }
        }
    }

    Ok(templates)
}

// NOTE: i wonder if there's a prettier way to write this function.
// the amount of `.clone()` bothers me
fn pretty_print(list: Vec<String>) {
    // ["a", "b", "c", "d", "e", "f", "g"] -> [["a", "b", "c"], ["d", "e", "f"], ["g"]]
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
    // [["a", "b", "c"], ["d", "e", "f"], ["g"]] -> [("a", "b", "c"), ("d", "e", "f"), ("g", "", "")]
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
