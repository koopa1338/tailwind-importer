use std::{io::Write, path::Path};

use walkdir::WalkDir;

/// Call this function in the `build.rs` of any component libraries
pub fn build_library() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("all.rs");
    let mut file = std::fs::File::create(&dest_path).unwrap();
    for entry in WalkDir::new("./src").into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            let source = std::fs::read_to_string(entry.path()).unwrap();
            file.write_all(source.as_bytes()).unwrap();
        }
    }
}

/// Call this function in the `build.rs` of the main application
pub fn build_frontend(output: impl AsRef<Path>, input: impl AsRef<Path>, minify: bool) {
    let target = std::env::var_os("OUT_DIR").unwrap();
    let target_dir = Path::new(&target).parent().unwrap().parent().unwrap();
    let mut content = vec![format!("{}/**/*.rs", target_dir.display())];

    // Merge additional content options if a tailwind config file is present
    let root = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let tailwind_config = Path::new(&root).join("tailwind.config.js");
    if tailwind_config.exists() {
        let config = std::fs::read_to_string(tailwind_config).unwrap();
        let content_opt = config.find("content:").unwrap();
        let start = config[content_opt..].find('[').unwrap() + content_opt + 1;
        let end = config[content_opt..].find(']').unwrap() + content_opt;
        let config_content = config[start..end]
            .split(',')
            .map(|s| {
                let mut option = String::from(s);
                option.retain(|c| !c.is_whitespace());
                option
            })
            .collect::<Vec<_>>();
        content.extend_from_slice(&config_content);
        content.dedup();
    }

    let conten_arg = content.join(",");
    let out = &output.as_ref().display().to_string();
    let inp = &input.as_ref().display().to_string();
    let mut args = vec!["--content", &conten_arg, "-o", &out];
    if minify {
        args.push("--minify");
    }
    std::process::Command::new("npx")
        .arg("tailwindcss")
        .arg("-i")
        .arg(&inp)
        .args(&args)
        .output()
        .expect("failed to execute process");

    println!("cargo:rerun-if-changed=./src");
}
