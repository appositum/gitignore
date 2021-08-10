mod api;
pub use api::{get_templates, get_template_list};

mod error;
pub use error::GIError;

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
