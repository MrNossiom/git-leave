use crate::log::{println_label, OutputLabel};
use git2::{
	Branch, BranchType, Cred, Error as GitError, PushOptions, RemoteCallbacks, Repository, Status,
};
use std::io::{stdout, Write};

pub fn is_repo_dirty(repo: &Repository) -> bool {
	if let Ok(statuses) = repo.statuses(None) {
		for status in statuses.iter() {
			match status.status() {
				Status::IGNORED => continue,
				_ => {
					return true;
				}
			}
		}
	}

	false
}

pub fn find_ahead_branches_in_repo(repo: &Repository) -> Vec<Branch> {
	// Iterate over all local branches
	// For each, check is a branch is ahead of its remote counterpart

	// Get all local branches
	let local_branches = repo
		.branches(Some(BranchType::Local))
		.expect("Could not get local branches")
		.map(|b| b.unwrap().0)
		.collect::<Vec<Branch>>();

	let mut ahead_branches: Vec<Branch> = Vec::new();

	// Iterate over all local branches
	for branch in local_branches {
		if let Ok(remote_branch) = branch.upstream() {
			let (last_local_commit, last_remote_commit) = (
				branch
					.get()
					.peel_to_commit()
					.expect("could not get last commit on local branch"),
				remote_branch
					.get()
					.peel_to_commit()
					.expect("could not get last commit on remote branch"),
			);

			if repo
				.graph_descendant_of(last_local_commit.id(), last_remote_commit.id())
				.expect("could not get graph difference between commits")
			{
				ahead_branches.push(branch)
			}
		} else {
			println_label(
				OutputLabel::Info("Info"),
				format!(
					"No upstream branch for {} in {}",
					branch.name().unwrap().unwrap(),
					repo.path().parent().unwrap().to_str().unwrap()
				),
			);
		}
	}

	ahead_branches
}

pub fn find_remote_and_push(repo: &Repository) -> Result<(), GitError> {
	let mut remote_callbacks = RemoteCallbacks::new();
	remote_callbacks.credentials(|_url, username_from_url, _allowed_types| {
		Cred::ssh_key(
			username_from_url.unwrap(),
			None,
			std::path::Path::new(&format!("{}/.ssh/id_rsa", env!("HOME"))),
			None,
		)
	});

	remote_callbacks.sideband_progress(|data| {
		print!("remote: {}", std::str::from_utf8(data).unwrap());
		stdout().flush().unwrap();
		true
	});

	remote_callbacks.transfer_progress(|stats| {
		if stats.received_objects() == stats.total_objects() {
			print!(
				"Resolving deltas {}/{}\r",
				stats.indexed_deltas(),
				stats.total_deltas()
			);
		} else if stats.total_objects() > 0 {
			print!(
				"Received {}/{} objects ({}) in {} bytes\r",
				stats.received_objects(),
				stats.total_objects(),
				stats.indexed_objects(),
				stats.received_bytes()
			);
		}
		stdout().flush().unwrap();
		true
	});

	remote_callbacks.update_tips(|ref_name, a, b| {
		if a.is_zero() {
			println!("[new]     {:20} {}", b, ref_name);
		} else {
			println!("[updated] {:10}..{:10} {}", a, b, ref_name);
		}
		true
	});

	let mut remote = match repo.find_remote("origin") {
		Ok(remote) => remote,
		Err(_) => {
			println_label(
				OutputLabel::Warning,
				format!(
					"No remote named origin found in {}",
					repo.path().parent().unwrap().to_str().unwrap()
				),
			);

			repo.find_remote(repo.remotes().unwrap().iter().next().unwrap().unwrap())
				.unwrap()
		}
	};

	let mut push_options = PushOptions::new();
	push_options.remote_callbacks(remote_callbacks);

	remote.push(&[] as &[&str], Some(&mut push_options))
}
