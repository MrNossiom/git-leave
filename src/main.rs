mod log;
mod utils;

use clap::Parser;
use git2::{Branch, Repository};
use log::{println, println_label, OutputLabel};
use std::{path::Path, time::Instant};
use utils::{
	crawl::crawl_directory_for_repos,
	git::{find_ahead_branches_in_repo, is_repo_dirty},
};
use yansi::Paint;

/// Push all commits in git repositories
#[derive(Parser, Debug)]
#[clap(name = "git-leave", about, version, author, long_about = None)]
struct Arguments {
	/// The directory to search in
	#[clap(default_value_t = String::from("."))]
	directory: String,

	/// Push commits to remote
	#[clap(short, long)]
	push: bool,

	/// Don't trim output
	#[clap(short, long)]
	notrim: bool,
}

fn main() {
	// Enable coloring on Windows if possible
	#[cfg(windows)]
	if !Paint::enable_windows_ascii() {
		Paint::disable();
	}

	// Parse command line arguments
	let args = Arguments::parse();

	// Display the name of the program
	println_label(
		OutputLabel::Success("Welcome"),
		format!("to {}", Paint::yellow("git-leave")),
	);

	// Get absolute path
	let search_directory = match Path::new(&args.directory).canonicalize() {
		Ok(path) => path,
		Err(err) => {
			println_label(
				OutputLabel::Error,
				format!(
					"Could not get absolute path of specified directory: {}",
					err
				),
			);

			return;
		}
	};

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let repos = match crawl_directory_for_repos(&search_directory) {
		Ok(repos) => repos,
		Err(err) => {
			println_label(
				OutputLabel::Error,
				format!(
					"Something went wrong while trying to crawl the directory: {}",
					err
				),
			);

			return;
		}
	};

	// Exit if no git repositories were found
	if repos.is_empty() {
		println_label(OutputLabel::Info("Empty"), "No git repositories found");

		return;
	}

	println_label(
		OutputLabel::Info("Found"),
		format!(
			"{} repositories in {}s",
			&repos.len(),
			begin_search_time.elapsed().as_millis() as f64 / 1000.0
		),
	);

	// Check if there are dirty repositories
	let dirty_repos: Vec<&Repository> = repos.iter().filter(|repo| is_repo_dirty(repo)).collect();

	if !dirty_repos.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!("{} dirty repositories", &dirty_repos.len()),
		);

		dirty_repos.iter().for_each(|repo| {
			println(
				repo.path()
					.parent()
					.unwrap()
					.to_str()
					.unwrap()
					.replace(env!("HOME"), "~"),
			);
		});
	}

	// Check if a repo has any local ahead branch
	let repos_with_ahead_branches: Vec<(&Repository, Vec<Branch>)> = repos
		.iter()
		.map(|repo| (repo, find_ahead_branches_in_repo(repo)))
		.filter(|vec| !vec.1.is_empty())
		.collect();

	if !repos_with_ahead_branches.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!(
				"{} repositories that have not pushed commits to remote",
				&repos_with_ahead_branches.len()
			),
		);

		repos_with_ahead_branches
			.iter()
			.for_each(|(repo, ahead_branches)| {
				println(format!(
					"Repository {} have these branches ahead: {}",
					Paint::yellow(
						repo.path()
							.parent()
							.unwrap()
							.file_name()
							.unwrap()
							.to_string_lossy()
					),
					Paint::yellow(
						ahead_branches
							.iter()
							.map(|branch| branch.name().unwrap().unwrap_or("<no name found>"))
							.collect::<Vec<&str>>()
							.join("/")
					)
				));
			});
	}
}
