use std::{fs, process::Command};

fn main() {
    // Install Node Dependencies
    let output = Command::new("sh")
        .arg("-c")
        .arg("npm clean-install")
        .output()
        .unwrap();

    // Ensure Node Install worked
    if !output.status.success() {
        panic!(
            "Shell command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Create Assets directory
    fs::create_dir_all("assets/scripts").unwrap();

    // Generate TailwindCSS file
    let output = Command::new("sh")
        .arg("-c")
        .arg("npx @tailwindcss/cli -i ./tailwind.css -o ./assets/css/main.css")
        .output()
        .unwrap();

    // Ensure TailwindCSS worked
    if !output.status.success() {
        panic!(
            "Shell command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Populate Assets directory with HTMX
    fs::copy(
        "node_modules/htmx.org/dist/htmx.js",
        "assets/scripts/htmx.js",
    )
    .unwrap();
}
