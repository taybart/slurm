use clap::Parser;
use git2::{Cred, RemoteCallbacks};
use std::env;
use std::path::{Path, PathBuf};
use url::Url;

mod cli;

fn get_remote_callbacks(identity_file: Option<PathBuf>) -> RemoteCallbacks<'static> {
    let file = identity_file.unwrap_or(
        Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())).to_path_buf(),
    );
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::ssh_key(
            "git", // TODO: github only
            None, &file, None,
        )
    });
    callbacks
}

// clone respository with ssh authentication
fn clone(
    repo: &String,
    dest: &PathBuf,
    identity_file: Option<PathBuf>,
) -> Result<git2::Repository, git2::Error> {
    // Prepare callbacks.

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(get_remote_callbacks(identity_file));

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    builder.clone(repo, Path::new(&dest))
}

fn parse_slugs(
    repo: &git2::Repository,
    slugs: Vec<&str>,
    identity_file: Option<PathBuf>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut remote = repo.find_remote("origin").unwrap();

    // Connect to the remote and call the printing function for each of the
    // remote references.
    let connection = remote
        .connect_auth(
            git2::Direction::Fetch,
            Some(get_remote_callbacks(identity_file)),
            None,
        )
        .unwrap();

    let rest = slugs[3..].join("/");
    // Get the list of references on the remote and print out their name next to
    // what they point to.
    for head in connection.list().unwrap().iter() {
        if head.name().starts_with("refs/heads/") {
            //     println!("{}", head.name().trim_start_matches("refs/heads/"));

            let br = head.name().trim_start_matches("refs/heads/");
            if rest.starts_with(br) {
                let file = slugs[3..]
                    .join("/")
                    .trim_start_matches(&format!("{}/", br))
                    .to_string();
                return Ok((br.to_string(), file));
            }
        }
    }
    Err(Box::from("could not determine branch"))
}

fn validate_url(input: &String) -> Result<(Url, String), String> {
    let url = Url::parse(input).map_err(|e| format!("could not parse url {}", e))?;
    // TODO: github only
    if url.host_str() != Some("github.com") {
        if !url.has_host() {
            return Err("no domain in url".to_string());
        }
        return Err(format!("unkown domain {:?}", url.host_str()));
    }

    let slugs: Vec<&str> = url.path_segments().unwrap().collect();

    if slugs[2] != "tree" && slugs[2] != "blob" {
        // TODO: github only
        return Err("malformed url, expected tree or blob in path name".to_string());
    }

    let repo_name = format!("https://github.com/{}", slugs[0..2].join("/")); // TODO: github only

    Ok((url.clone(), repo_name))
}

fn cleanup(repo: &PathBuf) -> Result<(), String> {
    println!("cleanup temp directory {}", repo.display());
    std::fs::remove_dir_all(repo).map_err(|e| format!("remove tmp dir {}", e))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    let mut dest = env::temp_dir();
    dest.push("slurmy");

    let (url, repo_name) = validate_url(&cli.get.to_string())?;

    println!("getting repo contents...");

    let repo = clone(&repo_name, &dest, cli.identity_file.to_owned())
        .map_err(|e| format!("could not clone repository {}", e))?;

    let (branch, file) = parse_slugs(
        &repo,
        url.path_segments().unwrap().collect(),
        cli.identity_file,
    )?;
    println!("{},{}", branch, file);

    println!("checking out branch...");
    let (object, reference) = repo
        .revparse_ext(&format!("origin/{}", branch))
        .expect("Object not found");
    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");

    println!("getting files...");

    // TODO: replace this with non-external/non-platspecific commands
    std::process::Command::new("mv")
        .args(&[
            // FIXME: should not use format
            format!("{}/{}", dest.as_path().to_str().unwrap(), file).as_str(),
            format!("./{}", file.split("/").last().unwrap()).as_str(),
        ])
        .spawn()
        .unwrap();

    cleanup(&dest)?;
    Ok(())
}
