use std::{fs, io::Error};

use serde::Deserialize;

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
pub struct UploadParams {
    server: String,
    owner: String,
    repo: String,
    commit: String,
    filename: String,
}

pub fn create_file(params: UploadParams) -> Result<(), Error> {
    let dir = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, params.server, params.owner, params.repo, params.commit
    );
    let path = format!("{}/{}", dir, params.filename);

    match fs::create_dir_all(dir) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    let file = fs::File::create(path);
    match file {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
