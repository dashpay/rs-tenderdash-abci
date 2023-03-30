use std::{
    env,
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::constants::DEFAULT_TENDERDASH_COMMITISH;

/// Check out a specific commitish of the tenderdash repository.
///
/// As this tool is mainly used by build.rs script, we rely
/// on cargo to decide wherther or not to call it. It means
/// we will not be called too frequently, so the fetch will
/// not happen too often.
pub fn fetch_commitish(tenderdash_dir: &Path, url: &str, commitish: &str) {
    println!(
        "  [info] => Cloning {} into {} folder",
        url,
        tenderdash_dir.join("tenderdash").to_string_lossy()
    );
    // std::env::set_current_dir(workspace_dir).expect("cannot change directory to
    // root dir");

    // We use `git` executable as we need --depth option, not supported by git2
    // crate
    exec(
        std::process::Command::new("git")
            .arg("-C")
            .arg(tenderdash_dir)
            .arg("fetch")
            .arg("--depth=1")
            .arg("origin")
            .arg(commitish),
    );

    exec(
        std::process::Command::new("git")
            .arg("-C")
            .arg(tenderdash_dir)
            .arg("checkout")
            .arg(commitish),
    );
}

/// Execute the command, panic on any error
fn exec(cmd: &mut std::process::Command) {
    let output = cmd.output().expect("command execution failed");

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("git command {:?} failed: {}", cmd, output.status);
    }
}
/// Copy generated files to target folder
pub fn copy_files(src_dir: &Path, target_dir: &Path) {
    // Remove old compiled files
    remove_dir_all(target_dir).unwrap_or_default();
    create_dir_all(target_dir).unwrap();

    // Copy new compiled files (prost does not use folder structures)
    let errors = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            copy(
                e.path(),
                std::path::Path::new(&format!(
                    "{}/{}",
                    &target_dir.display(),
                    &e.file_name().to_os_string().to_str().unwrap()
                )),
            )
        })
        .filter_map(|e| e.err())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        for e in errors {
            println!("[error] => Error while copying compiled file: {e}");
        }
        panic!("[error] => Aborted.");
    }
}

/// Walk through the list of directories and gather all *.proto files
pub fn find_proto_files(proto_paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut protos: Vec<PathBuf> = vec![];
    for proto_path in &proto_paths {
        protos.append(
            &mut WalkDir::new(proto_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file()
                        && e.path().extension().is_some()
                        && e.path().extension().unwrap() == "proto"
                })
                .map(|e| e.into_path())
                .collect(),
        );
    }
    protos
}

pub fn abci_version<T: AsRef<Path>>(dir: T) -> String {
    let mut file_path = dir.as_ref().to_path_buf();
    file_path.push("version/version.go");

    let contents = read_to_string(&file_path).expect("cannot read version/version.go");
    use regex::Regex;

    let re = Regex::new(r##"(?m)^\s+ABCISemVer\s*=\s*"([^"]+)"\s+*$"##).unwrap();
    let captures = re
        .captures(&contents)
        .expect("cannot find ABCISemVer in version/version.go");

    captures
        .get(1)
        .expect("ABCISemVer not found in version/version.go")
        .as_str()
        .to_string()
}

/// Create tenderdash.rs with library information
pub fn generate_tenderdash_lib(prost_dir: &Path, tenderdash_lib_target: &Path, abci_version: &str) {
    let mut file_names = WalkDir::new(prost_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name().to_str().unwrap().starts_with("tendermint.")
                && e.file_name().to_str().unwrap().ends_with(".rs")
        })
        .map(|d| d.file_name().to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    file_names.sort();

    let mut content =
        String::from("//! Tenderdash-proto auto-generated sub-modules for Tenderdash\n");
    let tab = "    ".to_string();

    for file_name in file_names {
        let parts: Vec<_> = file_name
            .strip_prefix("tendermint.")
            .unwrap()
            .strip_suffix(".rs")
            .unwrap()
            .split('.')
            .rev()
            .collect();

        let mut tab_count = parts.len();

        let mut inner_content = format!(
            "{}include!(\"prost/{}\");",
            tab.repeat(tab_count),
            file_name
        );

        for part in parts {
            tab_count -= 1;
            let tabs = tab.repeat(tab_count);
            //{tabs} pub mod {part} {
            //{inner_content}
            //{tabs} }
            inner_content = format!("{tabs}pub mod {part} {{\n{inner_content}\n{tabs}}}");
        }

        content = format!("{content}\n{inner_content}\n");
    }

    // Add meta
    content = format!(
        "{}
pub mod meta {{
    pub const REPOSITORY: &str = \"{}\";
    pub const COMMITISH: &str = \"{}\";
    /// Semantic version of ABCI protocol
    pub const ABCI_VERSION: &str = \"{}\";
    /// Version of Tenderdash server used to generate protobuf configs
    pub const TENDERDASH_VERSION: &str = env!(\"CARGO_PKG_VERSION\");
}}
",
        content,
        crate::constants::TENDERDASH_REPO,
        tenderdash_commitish(),
        abci_version,
    );

    let mut file =
        File::create(tenderdash_lib_target).expect("tenderdash library file create failed");
    file.write_all(content.as_bytes())
        .expect("tenderdash library file write failed");
}

pub(crate) fn tenderdash_commitish() -> String {
    match env::var("TENDERDASH_COMMITISH") {
        Ok(v) => v,
        Err(_) => DEFAULT_TENDERDASH_COMMITISH.to_string(),
    }
}
