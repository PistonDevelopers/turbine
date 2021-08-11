//! Read and write entities.

use crate::world::{self, World, ENTITY_COUNT};
use std::io;
use std::path::{Path, PathBuf};

/// The name of entity folder.
pub const FOLDER: &str = "entities";

/// Gets a list of all entity files.
pub fn files(project_folder: &str) -> io::Result<Vec<PathBuf>> {
    use std::fs::read_dir;

    let mut result = Vec::with_capacity(ENTITY_COUNT);
    let project_folder: PathBuf = PathBuf::from(project_folder);
    let entities_folder = project_folder.join(FOLDER);
    for entry in read_dir(entities_folder)? {
        let entry = entry?;
        // Ignore files that starts with ".".
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with('.') {
                continue;
            }
        }
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            result.push(entry.path());
        }
    }
    Ok(result)
}

/// Get entities folder.
pub fn folder(project_folder: &str) -> PathBuf {
    let project_folder: PathBuf = PathBuf::from(project_folder);
    project_folder.join(FOLDER)
}

/// Loads entities from files.
pub fn load(w: &mut World, files: &[PathBuf]) -> Result<(), io::Error> {
    use crate::math::{Vec3, AABB};
    use piston_meta::bootstrap::Convert;
    use piston_meta::*;
    use range::Range;
    use std::fs::File;
    use std::io::Read;
    use std::sync::Arc;
    use world::Mask;

    fn read_header(mut convert: Convert) -> Result<(Range, (Option<Arc<String>>, usize)), ()> {
        let start = convert;
        let header = "header";
        let range = convert.start_node(header)?;
        convert.update(range);
        let mut name = None;
        let mut entity_id = None;
        loop {
            if let Ok((range, val)) = convert.meta_string("name") {
                name = Some(val);
                convert.update(range);
            } else if let Ok((range, val)) = convert.meta_f64("entity_id") {
                entity_id = Some(val as usize);
                convert.update(range);
            } else if let Ok(range) = convert.end_node(header) {
                convert.update(range);
                break;
            } else {
                return Err(());
            }
        }
        let entity_id = match entity_id {
            None => {
                return Err(());
            }
            Some(x) => x,
        };
        Ok((convert.subtract(start), (name, entity_id)))
    }

    fn read_vec3(name: &str, mut convert: Convert) -> Result<(Range, Vec3), ()> {
        let start = convert;
        let range = convert.start_node(name)?;
        convert.update(range);
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        loop {
            if let Ok((range, val)) = convert.meta_f64("x") {
                x = val as f32;
                convert.update(range);
            } else if let Ok((range, val)) = convert.meta_f64("y") {
                y = val as f32;
                convert.update(range);
            } else if let Ok((range, val)) = convert.meta_f64("z") {
                z = val as f32;
                convert.update(range);
            } else if let Ok(range) = convert.end_node(name) {
                convert.update(range);
                break;
            } else {
                return Err(());
            }
        }

        Ok((convert.subtract(start), [x, y, z]))
    }

    fn read_aabb(mut convert: Convert) -> Result<(Range, AABB), ()> {
        let start = convert;
        let name = "aabb";
        let range = convert.start_node(name)?;
        convert.update(range);
        let mut min = None;
        let mut max = None;
        loop {
            if let Ok((range, val)) = read_vec3("min", convert) {
                min = Some(val);
                convert.update(range);
            } else if let Ok((range, val)) = read_vec3("max", convert) {
                max = Some(val);
                convert.update(range);
            } else if let Ok(range) = convert.end_node(name) {
                convert.update(range);
                break;
            } else {
                return Err(());
            }
        }
        let min = match min {
            None => {
                return Err(());
            }
            Some(x) => x,
        };
        let max = match max {
            None => {
                return Err(());
            }
            Some(x) => x,
        };
        Ok((convert.subtract(start), AABB { min, max }))
    }

    let syntax_source = include_str!("../../assets/entity/syntax.txt");
    let syntax = stderr_unwrap(syntax_source, syntax(syntax_source));
    let mut data = vec![];
    let mut entity_source = String::new();
    for f in files {
        data.clear();
        entity_source.clear();

        let mut file = File::open(f)?;
        file.read_to_string(&mut entity_source)?;

        // TODO: Return an error message.
        stderr_unwrap(&entity_source, parse(&syntax, &entity_source, &mut data));

        let mut convert = Convert::new(&data);
        let mut header = None;
        let mut position = None;
        let mut aabb = None;
        loop {
            if let Ok((range, val)) = read_header(convert) {
                header = Some(val);
                convert.update(range);
            } else if let Ok((range, val)) = read_vec3("position", convert) {
                position = Some(val);
                convert.update(range);
            } else if let Ok((range, val)) = read_aabb(convert) {
                aabb = Some(val);
                convert.update(range);
            } else {
                break;
            }
        }

        // TODO: Return an error message if header is missing.
        let (name, id) = match header {
            None => {
                panic!("header is missing in file `{}`", f.to_str().unwrap());
            }
            Some(x) => x,
        };
        w.mask[id].insert(Mask::ALIVE);
        w.name[id] = name;
        if let Some(pos) = position {
            w.init.position[id] = pos;
            w.prev.position[id] = pos;
            w.current.position[id] = pos;
            w.next.position[id] = pos;
        }
        if let Some(aabb) = aabb {
            w.aabb[id] = aabb;
            w.mask[id].insert(world::Mask::AABB);
        }

        info!("Loaded entity {}", f.file_name().unwrap().to_str().unwrap());
    }
    Ok(())
}

/// Saves entities data.
pub fn save<P: AsRef<Path>>(w: &World, entities_folder: P) -> Result<(), io::Error> {
    use std::fs::File;
    use std::io::Write;
    use world::Mask;

    let entities_folder = entities_folder.as_ref();
    for id in 0..ENTITY_COUNT {
        if !w.mask[id].contains(Mask::ALIVE) {
            continue;
        }

        let f = entities_folder.join(format!("{}.txt", id));
        let mut file = File::create(f)?;

        // Header
        if let Some(ref name) = w.name[id] {
            writeln!(file, "{}", name)?;
        } else {
            writeln!(file, "<name>")?;
        }
        writeln!(file, "{}", id)?;

        // Position.
        writeln!(file, "position")?;
        let pos = w.init.position[id];
        for i in 0..3 {
            writeln!(file, "\t{}", pos[i])?;
        }

        // AABB.
        if w.mask[id].contains(world::Mask::AABB) {
            let aabb = w.aabb[id];
            writeln!(file, "aabb")?;
            writeln!(file, "  min")?;
            for i in 0..3 {
                writeln!(file, "    {}", aabb.min[i])?;
            }
            writeln!(file, "  max")?;
            for i in 0..3 {
                writeln!(file, "    {}", aabb.max[i])?;
            }
        }

        info!("Saved entity id {}", id);
    }
    Ok(())
}
