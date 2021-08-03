use std::env;
use reqwest::header::USER_AGENT;
use serde_json::{self, Value as JsonValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // skip filename
    let args: Vec<String> = env::args().skip(1).collect();

    let api = String::from("https://api.github.com/gitignore/templates");
    let client = reqwest::Client::new();

    // client consumes string
    let templates_all: String = client.get(api.clone())
        .header(USER_AGENT, "rust cli tool")
        .send().await?
        .text().await?;

    let templates_data: JsonValue = serde_json::from_str(&templates_all[..])?;

    if !args.is_empty() {
        if let JsonValue::Array(ts) = templates_data {
            for a in &args {
                if !ts.contains(&JsonValue::String(a.to_string())) {
                    eprintln!("Couldn't find template {}", a);
                    return Ok(());
                }
            }
        }

        let urls = args.iter().map(|arg| format!("{}/{}", api, arg));

        let bodies: Vec<_> = urls.map(|url| {
            let client = client.clone();

            tokio::spawn(async move {
                client.get(url)
                    .header(USER_AGENT, "rust cli tool")
                    .send().await?
                    .text().await
            })
        }).collect();

        let mut templates: Vec<JsonValue> = Vec::new();

        for body in bodies {
            match body.await {
                Err(e) => {
                    eprintln!("tokio error = {}", e);
                    return Ok(());
                },
                Ok(Err(e)) => {
                    eprintln!("reqwest error = {}", e);
                    return Ok(());
                },
                Ok(Ok(b)) => {
                    let template: JsonValue = serde_json::from_str(&b[..])?;
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

    } else if let JsonValue::Array(templates) = templates_data {
        for t in templates {
            if let JsonValue::String(s) = t {
                println!("{}", s);
            }
        }
    }

    Ok(())
}
