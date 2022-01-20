use crate::log::{print_label, println_label, OutputLabel};
use std::io::{stdin, stdout, Write};

#[allow(dead_code)]
pub enum AskDefault {
	Yes,
	No,
	None,
}

pub fn ask(question: &str, default: AskDefault) -> bool {
	let ask_case = match default {
		AskDefault::Yes => "[Y/n]",
		AskDefault::No => "[y/N]",
		AskDefault::None => "[y/n]",
	};

	print_label(OutputLabel::Prompt(ask_case), question);
	stdout().flush().unwrap();

	let mut input = String::new();
	stdin().read_line(&mut input).unwrap();

	match input.as_str() {
		"Y\n" => true,
		"y\n" => true,
		"N\n" => false,
		"n\n" => false,
		"\n" => match default {
			AskDefault::Yes => true,
			AskDefault::No => false,
			AskDefault::None => {
				println_label(OutputLabel::Error, "Please answer with yes or no");
				ask(question, default)
			}
		},
		_ => {
			println_label(OutputLabel::Error, "Please answer with yes or no");
			ask(question, default)
		}
	}
}
