extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::io::Read;
use std::error::Error;

fn main() {
    initialize_files();

    let mut alternatives = load_alternatives();
}

fn initialize_files() {
    match std::fs::create_dir_all("/etc/alternatives") {
        Ok(_) => (),
        Err(e) => panic!("update-alternatives: could not create directory \
                         /etc/alternatives: {}", e.description()),
    }
}

fn load_alternatives() -> AlternativeMap {
    let entries = match std::fs::read_dir("/etc/alternatives") {
        Ok(i) => i,
        Err(e) => panic!("update-alternatives: could not read directory \
                         /etc/alternatives: {}", e.description()),
    };

    let mut alternatives = AlternativeMap::new(); 

    for maybe_entry in entries {
        let entry = match maybe_entry {
            Ok(p) => p,
            Err(e) => panic!("update-alternatives could not read entry in \
                             directory /etc/alternatives: {}", e.description()),
        };

        let path = entry.path();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => panic!("update-alternatives: could not read metadata of \
                             file {}: {}", path.to_string_lossy(),
                             e.description()),
        };

        if !metadata.file_type().is_file() {
            continue;
        }

        match path.extension() {
            Some(e) => match e.to_string_lossy().as_ref() {
                "json" => (),
                _ => continue,
            },
            None => continue,
        }

        let name = path.file_name().unwrap(); // should have a name
        let key = String::from(name.to_string_lossy());

        let parsed = parse_alternatives_file(&path);

        println!("found link {} with {} alternatives",
                 parsed.name, parsed.alternatives.len());

        alternatives.insert(key, parsed);
    }

    return alternatives;
}

// path must be a normal file
fn parse_alternatives_file(path: &std::path::Path) -> Alternatives {
    let name = String::from(path.file_name().unwrap().to_string_lossy());

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => panic!("update-alternatives: unable to open file {}: {}",
                         path.to_string_lossy(), e.description()),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(e) => panic!("update-alternatives: unable to read file {}: {}",
                         path.to_string_lossy(), e.description()),
        _ => (),
    }

    match serde_json::from_str(&contents) {
        Ok(a) => {
            let alternatives: Alternatives = a;

            if alternatives.name != name {
                panic!("update-alternatives: error parsing file {}: filename \
                       does not match name", path.to_string_lossy());
            }

            alternatives
        },
        Err(e) => panic!("update-alternatives: unable to parse file {}: {}",
                         path.to_string_lossy(), e.description()),
    }
}

type AlternativeMap = std::collections::HashMap<String, Alternatives>;

#[derive(Serialize, Deserialize)]
struct Alternative {
    name: String,
    target: std::path::PathBuf,
    location: std::path::PathBuf,
    priority: i32,
}

#[derive(Serialize, Deserialize)]
struct Alternatives {
    name: String,
    alternatives: Vec<Alternative>,
}
