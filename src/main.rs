use clap::{App, load_yaml};
use gitignore::{self as gi, GIError};
use reqwest as req;

#[tokio::main]
async fn main() -> Result<(), GIError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let client = req::Client::new();

    let all_templates: Vec<String> = gi::get_template_list(&client).await?;

    if matches.is_present("list") {
        gi::pretty_print(all_templates);

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

        gi::get_templates(&client, templates_input)
            .await?
            .into_iter()
            .for_each(|t| {
                println!("### {} ###\n{}", t.name, t.source);
            });
    }

    Ok(())
}
