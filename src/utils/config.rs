use clap::Parser;
use git2::Config as GitConfig;

#[derive(Parser)]
#[clap(name = "git-leave", about, version, author, long_about = None)]
pub struct Arguments {
	/// The directory to search in
	#[clap(default_value_t = String::from("."))]
	pub directory: String,

	/// Use git config default folder value for the directory to search in
	#[clap(short, long)]
	pub default: bool,
}

// Keys used in `.gitconfig` file
const CONFIG_KEY_DEFAULT_FOLDER: &str = "leaveTool.defaultFolder";

/// Contains all the parsed configuration keys for this tool
pub struct GitLeaveConfig {
	pub default_folder: Option<String>,
}

/// Return all config entries related to this tool
pub fn get_related_config() -> Option<GitLeaveConfig> {
	let config_path = match GitConfig::find_global() {
		Ok(path) => path,
		_ => return None,
	};

	let config = match GitConfig::open(&config_path) {
		Ok(config) => config,
		Err(err) => {
			eprintln!("Could not open global config: {}", err);

			return None;
		}
	};

	Some(GitLeaveConfig {
		default_folder: get_key_string_value(&config, CONFIG_KEY_DEFAULT_FOLDER),
	})
}

// Correctly parse string value for a given key
fn get_key_string_value(config: &GitConfig, key: &str) -> Option<String> {
	let string_value = match config.get_string(key) {
		Ok(value) => value,
		Err(_) => return None,
	};

	match string_value.as_str() {
		"" => None,
		string => Some(string.to_string()),
	}
}
