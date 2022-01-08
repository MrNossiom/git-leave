use crate::log::{pretty_output, println_label, OutputLabel};
use git2::Repository;
use std::{
	fs::read_dir,
	io::{self, stdin, stdout, Write},
	path::Path,
};

pub fn find_repos_in_dir(dir: &Path) -> io::Result<Vec<Repository>> {
	let mut repos: Vec<Repository> = Vec::new();

	// TODO: make this function more efficient (using threads)

	if dir.is_dir() {
		let dir_content = read_dir(dir)
			.expect("Couldn't read directory")
			.collect::<Vec<_>>();

		for entry in dir_content {
			let path = entry.expect("Couldn't read file or directory").path();

			if path.is_dir() {
				if let Ok(repo) = Repository::open(&path) {
					if repo.is_bare() {
						continue;
					}

					repos.push(repo);
				} else {
					find_repos_in_dir(&path)?;
				}
			}
		}
	}

	Ok(repos)
}

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

	print!(
		"{} ",
		pretty_output(OutputLabel::Prompt(ask_case), question).trim_end()
	);
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
