use std::process::Command;

fn main() {
	let _ = Command::new("npm").args(&["run", "build"]).current_dir("frontend/").status().unwrap();
}