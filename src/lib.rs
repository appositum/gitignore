mod api;
mod cli;
mod error;

use crate::error::GIError;

use std::{
    collections::HashMap,
    ops::Rem,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Fetches .gitignore templates from GitHub's API", long_about = None)]
pub struct Args {
    #[arg(short, long, action, help = "Requests list of all available templates")]
    list: bool,

    #[arg(
        required = true,
        conflicts_with = "list",
        num_args = 1..,
        value_delimiter = ' ',
        help = "Space separated list of templates. e.g: Rust C  Lua"
    )]
    templates: Vec<String>,

    #[arg(short, long, action, help = "Overwrites .gitignore file with output")]
    force: bool,

    #[arg(
        short,
        long,
        action,
        conflicts_with = "force",
        help = "Appends output to .gitignore file"
    )]
    append: bool,

    #[arg(short, long, action, help = "Overwrites .gitignore file with output")]
    output: Option<String>,
}

#[tokio::main]
pub async fn run() -> Result<(), GIError> {
    let args = Args::parse();

    let client = reqwest::Client::new();

    let all_templates: Vec<String> = api::get_template_list(&client).await?;

    if args.list {
        pretty_print(all_templates);

        return Ok(());
    }

    // we use this map to compare the lowercased user input to the lowercased template name.
    // this is necessary for case insensitivity because the templates name for the API endpoints
    // are case sensitive.
    let all_templates_map: HashMap<String, String> = all_templates
        .into_iter()
        .map(|t| (t.to_lowercase(), t))
        // {("adventuregamestudio", "AdventureGameStudio"), ("rust", "Rust"), ...}
        .collect();

    if !args.templates.is_empty() {
        let mut templates_not_found: Vec<String> = Vec::new();

        let templates_input: Vec<String> = args
            .templates
            .iter()
            .fold(vec![], |mut acc, template| {
                let template_lowercase = template.to_lowercase();

                match all_templates_map.get(&template_lowercase) {
                    None => templates_not_found.push(String::from(template)),
                    Some(template_found) => acc.push(template_found.clone()),
                }

                acc
            })
            .into_iter()
            .collect();

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

        if args.force {
            cli::flag_overwrite(output.clone());
            print_output = false;
        } else if args.append {
            cli::flag_append(output.clone());
            print_output = false;
        }

        if let Some(path) = args.output {
            cli::flag_output(output.clone(), path);
            print_output = false;
        }

        if print_output {
            println!("{}", output);
        }
    }

    Ok(())
}

fn pretty_print(input: Vec<String>) {
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
