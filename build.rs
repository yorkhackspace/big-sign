use std::process::Command;

fn main() {
	let _ = Command::new("npx").args(&["vite", "build"]).current_dir("frontend/").status().unwrap();
}