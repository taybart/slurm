use git2::{Cred, RemoteCallbacks};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use url::Url;

fn clone(repo: &String, dest: String) {
    // Prepare callbacks.
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    let _err = builder.clone(repo, Path::new(&dest));
}

fn cleanup(repo: String) {
    std::fs::remove_dir_all(repo).expect("remove git dir");
}

/*
 * TODO:
 * 1. cli -> gg https://...../files -
 * 2. parse url -> (repo, branch, file(s)
 * 3. clone
 * 4. mv selected file/folder to cwd
 * 5. cleanup
 */

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("https://github.com/taybart/dotfiles").unwrap();
    let dest = "./tmp";
    clone(&url.as_str().to_owned(), dest.to_string());
    println!("cloned {}", url);

    println!("{}", url);
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{:#?}", resp);
    cleanup(dest.to_string());
    Ok(())
}
