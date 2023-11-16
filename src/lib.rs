mod api;
mod cli;
mod error;

use error::GIError;

use std::ops::Rem;

use clap::{App, load_yaml};

#[tokio::main]
pub async fn run() -> Result<(), GIError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let client = reqwest::Client::new();

    let all_templates: Vec<String> = api::get_template_list(&client).await?;

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
            return Err(GIError::TemplateNotFound(templates_not_found));
        }

        let mut output = String::new();
        let mut print_output = true;

        api::get_templates(&client, templates_input)
            .await?
            .into_iter()
            .for_each(|t| {
                output.push_str(&format!("### {} ###\n{}", t.name, t.source));
            });

        if matches.is_present("force") {
            cli::flag_overwrite(output.clone());
            print_output = false;
        } else if matches.is_present("append") {
            cli::flag_append(output.clone());
            print_output = false;
        }

        if let Some(path) = matches.value_of("output") {
            cli::flag_output(output.clone(), path);
            print_output = false;
        }

        // idk about this lol
        if print_output {
            println!("{}", output);
        }
    }

    Ok(())
}

pub fn pretty_print(input: Vec<String>) {
    let mut list = input.clone();

    // add empty strings to make sure we can split into exactly 3 size chunks
    while list.len().rem(3) != 0 {
        list.push("".to_string());
    }

    let chunks: Vec<Vec<String>> = list
        .chunks(3)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|c| c.to_vec())
        .collect();

    // get length of the biggest string from subgroup
    let max1 = chunks
        .iter()
        .map(|subgroup| subgroup[0].len())
        .max()
        .unwrap();

    let max2 = chunks
        .iter()
        .map(|subgroup| subgroup[1].len())
        .max()
        .unwrap();

    chunks.iter().for_each(|chunk| {
        println!(
            "{:<w1$} {:<w2$} {}",
            chunk[0],
            chunk[1],
            chunk[2],
            w1 = max1,
            w2 = max2
        );
    })
}
