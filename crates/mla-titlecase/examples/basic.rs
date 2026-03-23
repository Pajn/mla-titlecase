#![allow(missing_docs)]

use mla_titlecase::{titlecase_mla, titlecase_with_options, TitleCaseOptions};

fn main() {
    println!("{}", titlecase_mla("the wind in the willows"));

    let options = TitleCaseOptions::with_protected_words(&["PostgreSQL"]);
    println!("{}", titlecase_with_options("learning postgresql with rust", &options));
}
