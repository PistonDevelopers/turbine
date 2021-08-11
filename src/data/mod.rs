//! Data.

use std::io;

pub mod entities;

/// Creates folders within project folders, such as entities folder.
pub fn create_folders(project_folder: &str) -> io::Result<()> {
    use std::fs::create_dir;
    use std::path::PathBuf;

    let project_folder: PathBuf = PathBuf::from(project_folder);
    if !project_folder.exists() {
        create_dir(&project_folder)?;
        info!("Created project folder");
    }
    let entities_folder = project_folder.join(entities::FOLDER);
    if !entities_folder.exists() {
        create_dir(&entities_folder)?;
        info!("Created entities folder");
    }

    Ok(())
}
