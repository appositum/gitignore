mod api;
mod cli;
mod error;

use crate::error::GIError;

use std::collections::HashMap;

use ansi_term::Colour::Red;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Fetches .gitignore templates from GitHub's API", long_about = None)]
pub struct Args {
    #[arg(
        required = true,
        conflicts_with_all = ["list", "search"],
        num_args = 1..,
        value_delimiter = ' ',
        help = "Space separated list of templates. e.g: Rust C  Lua"
    )]
    templates: Vec<String>,

    #[arg(short, long, action, help = "Requests list of all available templates")]
    list: bool,

    #[arg(
        short,
        long,
        action,
        conflicts_with_all = ["list", "force", "append", "output"],
        help = "Search for templates that match your string"
    )]
    search: Option<String>,

    #[arg(
        short,
        long,
        action,
        conflicts_with = "force",
        help = "Appends output to .gitignore file"
    )]
    append: bool,

    #[arg(short, long, action, help = "Overwrites .gitignore file with output")]
    force: bool,

    #[arg(
        short,
        long,
        action,
        help = "Redirects output to a file or stream (default: stdout)"
    )]
    output: Option<String>,
}

#[tokio::main]
pub async fn run() -> Result<(), GIError> {
    let args = Args::parse();

    let client = reqwest::Client::new();

    let all_templates: Vec<String> = api::get_template_list(&client).await?;

    if args.list {
        cli::flag_list(all_templates);
        return Ok(());
    }

    // we use this map to compare the lowercased user input to the lowercased template name.
    // this is necessary for case insensitivity because
    // the templates name for the API endpoints are case sensitive.
    let all_templates_map: HashMap<String, String> = all_templates
        .into_iter()
        .map(|t| (t.to_lowercase(), t))
        // {("adventuregamestudio", "AdventureGameStudio"), ("rust", "Rust"), ...}
        .collect();

    if let Some(search) = args.search {
        let search_lowercase = search.to_lowercase();

        for (k, v) in all_templates_map {
            if k.contains(&search_lowercase) {
                let matched: Vec<_> = k.match_indices(&search_lowercase).collect();
                let (index_start, _) = matched[0]; // only need the first substring match
                let index_end = index_start + search_lowercase.len();
                let matched_substr = &v[index_start..index_end];

                /*
                $ gitignore -s ara
                Laravel

                str_start      = Lar
                matched_substr = ara
                str_end        = vel
                */
                let (str_start, _rest) = v.split_at(index_start);
                let (_rest, str_end) = v.split_at(index_end);
                println!("{str_start}{}{str_end}", Red.paint(matched_substr));
            }
        }

        return Ok(());
    }

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
                output.push_str(&format!("### {} ###\n{}\n\n", t.name, t.source));
            });

        // remove the extra newline at end of string,
        // we only want two in between template sections.
        let _extra_newline = output.pop();

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
            print!("{}", output);
        }
    }

    Ok(())
}
