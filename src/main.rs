use ansi_term::Colour::{Green, Red};
use clap::{Arg, App};
use reqwest::header::USER_AGENT;
use serde_json::{self, Value as JsonValue};
use std::ops::Add;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: apparently, clap has support to read from a yaml file.
    // will try to use that in the future and avoid all this verbosity

    let matches = App::new("gitignore.rs")
        .version("0.1.0")
        .author("appositum")
        .about("Fetches .gitignore templates from GitHub's API")
        .arg(Arg::with_name("list")
             .help("Requests list of all available templates")
             .short("l")
             .long("list"))
        .arg(Arg::with_name("templates")
             .help("Comma separated list of templates. e.g.: Rust,Python,C")
             .index(1)
             .required(true)
             .conflicts_with("list"))

        // TODO
        .arg(Arg::with_name("file")
             .help("Overwrites .gitignore file with output")
             .short("f"))
        .arg(Arg::with_name("append")
             .help("Appends output to .gitignore file")
             .short("a")
             .long("append")
             .conflicts_with("file"))
        .arg(Arg::with_name("output")
             .help("Redirects output to a file or stream (default: stdout)")
             .short("o")
             .long("output"))
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
                .add(&format!("{}", Green.paint("--help"))[..]);

            eprintln!("{} Template(s) not found: {:?}\n\n{}",
                      Red.bold().paint("error:"),
                      templates_not_found,
                      usage);

            // TODO: make our own error wrapper type
            // so we can actually return Err() instead of this hack
            return Ok(());
        }

        let bodies: Vec<_> = urls.map(|url| {
            let client = client.clone();

            // TODO: use `request_body` instead of this block,
            // but types are mistmatching. getting rid of this repetition,
            // we can drop the `urls` variable and use `get_template` instead
            tokio::spawn(async move {
                println!("requesting {}", url);
                client.get(url)
                    .header(USER_AGENT, "gitignore.rs")
                    .send().await?
                    .text().await
            })
        }).collect();

        let mut templates: Vec<JsonValue> = Vec::new();

        for body in bodies {
            match body.await {
                Err(e) => {
                    eprintln!("{} {}", Red.bold().paint("tokio error:"), e.to_string());
                    return Ok(());
                },
                Ok(Err(e)) => {
                    eprintln!("{} {}", Red.bold().paint("reqwest error:"), e.to_string());
                    return Ok(());
                },
                Ok(Ok(b)) => {
                    let template = serde_json::from_str(&b[..])?;
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

async fn request_body(url: String, client: &reqwest::Client)
                      -> Result<String, reqwest::Error>
{
    client
        .get(url)
        .header(USER_AGENT, "gitignore.rs")
        .send().await?
        .text().await
}

async fn _get_template(name: String, client: &reqwest::Client)
                      -> Result<String, reqwest::Error>
{
    // NOTE: this looks disgusting. i define this api link in the main function
    // and pass it around as argument to the other request functions.
    // in this `get_template` case, i want to be able to pass only the name of the template
    // and have the api link be implicit, but it's inconsistent
    // with the way the other functions work. gonna figure out a better way
    // to design this. perhaps hardcode the link into the request_body function?
    let url = format!("https://api.github.com/gitignore/templates/{}", name);
    let body = request_body(url, client).await?;

    let data: JsonValue = serde_json::from_str(&body[..]).unwrap();
    let mut result = String::new();

    // TODO: make an error type to englobe both reqwest and serde errors
    if let JsonValue::String(name) = &data["name"] {
        if let JsonValue::String(source) = &data["source"] {
            result.push_str(
                &format!("### {} ###\n{}", name, source)[..]
            );
        }
    }

    Ok(result)
}


async fn get_all_templates(url: String, client: &reqwest::Client)
                           -> Result<Vec<String>, reqwest::Error>
{
    let body = request_body(url, client).await?;

    let mut result: Vec<String> = Vec::new();

    // TODO: same thing as previous TODO, make an error type for both
    if let JsonValue::Array(ts) = serde_json::from_str(&body[..]).unwrap() {
        for t in ts {
            if let JsonValue::String(s) = t {
                result.push(s);
            }
        }
    }

    Ok(result)
}
