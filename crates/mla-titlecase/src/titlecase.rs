use crate::config::TitleCaseOptions;
use crate::rules;
use crate::tokenizer::tokenize;

pub(crate) fn titlecase_mla(input: &str) -> String {
    titlecase_with_options(input, &TitleCaseOptions::default())
}

pub(crate) fn titlecase_with_options(input: &str, options: &TitleCaseOptions<'_>) -> String {
    let tokens = tokenize(input);
    rules::apply(&tokens, options)
}
