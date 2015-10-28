//! Read and write entities.

use std::io;
use std::path::PathBuf;
use world;

/// The name of entity folder.
pub const FOLDER: &'static str = "entities";

/// Gets a list of all entity files.
pub fn files(project_folder: &str) -> io::Result<Vec<PathBuf>> {
    use std::fs::read_dir;

    let mut result = Vec::with_capacity(world::ENTITY_COUNT);
    let project_folder: PathBuf = PathBuf::from(project_folder);
    let entities_folder = project_folder.join(FOLDER);
    for entry in try!(read_dir(entities_folder)) {
        let entry = try!(entry);
        let metadata = try!(entry.metadata());
        if metadata.is_file() {
            result.push(entry.path());
        }
    }
    Ok(result)
}
