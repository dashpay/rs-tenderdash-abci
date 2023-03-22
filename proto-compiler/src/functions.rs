use std::{
    env,
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    AutotagOption, Commit, FetchOptions, Oid, Reference, Repository,
};
use subtle_encoding::hex;
use walkdir::WalkDir;

use crate::constants::DEFAULT_TENDERDASH_COMMITISH;

/// Clone or open+fetch a repository and check out a specific commitish
/// In case of an existing repository, the origin remote will be set to `url`.
pub fn fetch_commitish(dir: &Path, url: &str, commitish: &str) {
    let repo = if dir.exists() {
        fetch_existing(dir, url)
    } else {
        clone_new(dir, url)
    };
    checkout_commitish(&repo, commitish)
}

fn clone_new(dir: &Path, url: &str) -> Repository {
    println!(
        "  [info] => Cloning {} into {} folder",
        url,
        dir.to_string_lossy()
    );

    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::All);
    fo.update_fetchhead(true);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);

    builder.clone(url, dir).unwrap()
}

fn fetch_existing(dir: &Path, url: &str) -> Repository {
    println!(
        "  [info] => Fetching from {} into existing {} folder",
        url,
        dir.to_string_lossy()
    );
    let repo = Repository::open(dir).unwrap();

    let mut fo = git2::FetchOptions::new();
    fo.download_tags(git2::AutotagOption::All);
    fo.update_fetchhead(true);

    let mut remote = repo
        .find_remote("origin")
        .unwrap_or_else(|_| repo.remote("origin", url).unwrap());
    if remote.url().is_none() || remote.url().unwrap() != url {
        repo.remote_set_url("origin", url).unwrap();
    }
    println!("  [info] => Fetching repo using remote `origin`");
    let specs: &[&str] = &[];
    remote.fetch(specs, Some(&mut fo), None).unwrap();

    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "  [info] => Received {}/{} objects in {} bytes (used {} local objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "  [info] => Received {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    Repository::open(dir).unwrap()
}

fn checkout_commitish(repo: &Repository, commitish: &str) {
    let (reference, commit) = find_reference_or_commit(repo, commitish);

    println!(
        "  [info] => Checking out repo in detached HEAD mode:\n    \
             [info] => id: {},\n    \
             [info] => author: {},\n    \
             [info] => committer: {},\n    \
             [info] => summary: {}",
        commit.id(),
        commit.author(),
        commit.committer(),
        commit.summary().unwrap_or(""),
    );

    match reference {
        None => repo.set_head_detached(commit.id()).unwrap(),
        Some(reference) => {
            println!("    [info] => name: {}", reference.shorthand().unwrap());
            repo.set_head(reference.name().unwrap()).unwrap();
        },
    }

    let mut checkout_options = CheckoutBuilder::new();
    checkout_options
        .force()
        .remove_untracked(true)
        .remove_ignored(true)
        .use_theirs(true);
    repo.checkout_head(Some(&mut checkout_options)).unwrap();
}

fn find_reference_or_commit<'a>(
    repo: &'a Repository,
    commitish: &str,
) -> (Option<Reference<'a>>, Commit<'a>) {
    let mut tried_origin = false; // we tried adding 'origin/' to the commitish

    let mut try_reference = repo.resolve_reference_from_short_name(commitish);
    if try_reference.is_err() {
        // Local branch might be missing, try the remote branch
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{commitish}"));
        tried_origin = true;
        if try_reference.is_err() {
            // Remote branch not found, last chance: try as a commit ID
            // Note: Oid::from_str() currently does an incorrect conversion and cuts the second half
            // of the ID. We are falling back on Oid::from_bytes() for now.
            let commitish_vec = hex::decode(commitish).unwrap_or_else(|_| {
                hex::decode_upper(commitish).expect(
                    "TENDERDASH_COMMITISH refers to non-existing or invalid git branch/tag/commit",
                )
            });
            return (
                None,
                repo.find_commit(Oid::from_bytes(commitish_vec.as_slice()).unwrap())
                    .unwrap(),
            );
        }
    }

    let mut reference = try_reference.unwrap();
    if reference.is_branch() {
        if tried_origin {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
        try_reference = repo.resolve_reference_from_short_name(&format!("origin/{commitish}"));
        reference = try_reference.unwrap();
        if reference.is_branch() {
            panic!("[error] => local branch names with 'origin/' prefix not supported");
        }
    }

    let commit = reference.peel_to_commit().unwrap();
    (Some(reference), commit)
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