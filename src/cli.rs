use std::collections::HashMap;
use std::fs::{
    self,
    File,
    OpenOptions,
};
use std::io::Write;
use std::ops::Rem;

use ansi_term::Colour::Red;

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

pub fn flag_search(search: String, templates: HashMap<String, String>) {
    let search_lowercase = search.to_lowercase();

    for (k, v) in templates {
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
