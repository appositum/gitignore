use ansi_term::Color::Red;
use reqwest as req;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::from_str as to_json;
use std::error;
use std::fmt::{self, Display};
use tokio::task;

#[derive(Deserialize, Debug)]
pub struct Template {
    pub name: String,
    pub source: String,
}

// only using this to deserialize the json array we get
// from requesting the first API endpoint with all templates
#[derive(Deserialize, Debug)]
struct TemplateList(Vec<String>);

#[derive(Debug)]
pub enum GIError {
    Json(serde_json::Error),
    Request(reqwest::Error),
    TaskJoin(task::JoinError),
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

impl From<task::JoinError> for GIError {
    fn from(err: task::JoinError) -> GIError {
        GIError::TaskJoin(err)
    }
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
                let template: Template = to_json(&b)?;
                templates.push(template);
            }
        }
    }

    Ok(templates)
}

// NOTE: i wonder if there's a prettier way to write this function.
// the amount of `.clone()` bothers me
pub fn pretty_print(list: Vec<String>) {
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
