use crate::analysis::TitleCaseAnalysis;
use crate::config::TitleCaseOptions;
use crate::rules;
use crate::tokenizer::tokenize;

pub(crate) fn titlecase_mla(input: &str) -> String {
    titlecase_with_options(input, &TitleCaseOptions::default())
}

pub(crate) fn titlecase_with_options(input: &str, options: &TitleCaseOptions<'_>) -> String {
    let tokens = tokenize(input);
    rules::apply(input, &tokens, options)
}

pub(crate) fn titlecase_into(out: &mut String, input: &str, options: &TitleCaseOptions<'_>) {
    out.clear();
    let tokens = tokenize(input);
    rules::apply_into(out, input, &tokens, options);
}

pub(crate) fn titlecase_analyze(input: &str, options: &TitleCaseOptions<'_>) -> TitleCaseAnalysis {
    let tokens = tokenize(input);
    rules::analyze(input, &tokens, options)
}
