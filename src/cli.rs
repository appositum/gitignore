use std::fs::{
    self,
    File,
    OpenOptions,
};
use std::io::Write;
use std::ops::Rem;

// TODO: this "pretty print" looks awful and unintuitive.
// The sorting is weird.
// Figure out a way to make it better.
pub fn flag_list(input: Vec<String>) {
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

pub fn flag_append(text: String) {
    let mut append = true;

    if fs::exists(".gitignore").unwrap() {
        if fs::read_to_string(".gitignore").unwrap().is_empty() {
            append = false;
        }
    } else {
        append = false;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(".gitignore")
        .unwrap();

    let formatted_text = if append { format!("\n{}", text) } else { text };

    file.write_all(formatted_text.as_bytes()).unwrap();
}

pub fn flag_output(text: String, filename: String) {
    let mut file = File::create(filename).unwrap();

    file.write_all(text.as_bytes()).unwrap();
}

pub fn flag_overwrite(text: String) {
    flag_output(text, String::from(".gitignore"))
}
