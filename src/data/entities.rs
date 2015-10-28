//! Read and write entities.

use std::io;
use std::path::PathBuf;
use world::{ ENTITY_COUNT, World };

/// The name of entity folder.
pub const FOLDER: &'static str = "entities";

/// Gets a list of all entity files.
pub fn files(project_folder: &str) -> io::Result<Vec<PathBuf>> {
    use std::fs::read_dir;

    let mut result = Vec::with_capacity(ENTITY_COUNT);
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

/// Loads entities from files.
pub fn load(w: &mut World, files: &[PathBuf]) -> Result<(), io::Error> {
    use std::fs::File;
    use std::io::Read;
    use piston_meta::*;

    let syntax_source = include_str!("../../assets/entity/syntax.txt");
    let syntax = stderr_unwrap(syntax_source, syntax(syntax_source));
    let mut data = vec![];
    let mut entity_source = String::new();
    for f in files {
        data.clear();
        entity_source.clear();

        let mut file = try!(File::open(f));
        try!(file.read_to_string(&mut entity_source));

        // TODO: Return an error message.
        stderr_unwrap(&entity_source, parse(&syntax, &entity_source, &mut data));

        // TEST
        json::print(&data);

        info!("Loaded entity {}", f.file_name().unwrap().to_str().unwrap());
    }
    Ok(())
}
