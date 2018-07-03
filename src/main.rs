extern crate clap;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::io::Write;
use std::io::Read;
use std::error::Error;

fn main() {
    let matches = clap::App::new("update-alternatives")
        .version("0.1.0")
        .author("Gregory Meyer <gregjm@umich.edu>")
        .about("handles symlinking for multiple files")
        .subcommand(clap::SubCommand::with_name("list")
                        .about("list alternatives for a given link")
                        .arg(clap::Arg::with_name("LINK")
                                 .help("the link to query")
                                 .required(true)
                                 .index(1)))
        .subcommand(clap::SubCommand::with_name("add")
                        .about("add a link")
                        .arg(clap::Arg::with_name("TARGET")
                                 .help("the target of the link to add")
                                 .required(true)
                                 .index(1))
                        .arg(clap::Arg::with_name("NAME")
                                 .help("the name of the link to add")
                                 .required(true)
                                 .index(2))
                        .arg(clap::Arg::with_name("WEIGHT")
                                 .help("the weight of the link to add")
                                 .required(true)
                                 .index(3)))
        .get_matches();

    initialize_files();

    let mut alternatives = load_alternatives();
    let mut mutated = false;

    if let Some(list_matches) = matches.subcommand_matches("list") {
        let link = list_matches.value_of("LINK").unwrap();

        if alternatives.contains_key(link) {
            println!("update-alternatives: listing alternatives for {}...",
                     link);
        } else {
            panic!("update-alternatives: link {} does not exist", link);
        }
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        let target = add_matches.value_of("TARGET").unwrap();
        let name = add_matches.value_of("NAME").unwrap();
        let weight_str = add_matches.value_of("WEIGHT").unwrap();

        let weight: i32 = match weight_str.parse() {
            Ok(w) => w,
            Err(e) => panic!("update-alternatives: could not parse {} as \
                             weight: {}", weight_str, e.description()),
        };

        add_alternative(&mut alternatives, target, name, weight);
    }

    if mutated {
        commit_alternatives(&alternatives);
    }
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

fn add_alternative(alternatives: &mut AlternativeMap, target: &str,
                   name: &str, weight: i32) {
    let target_path = std::path::PathBuf::from(target);
    let location = std::path::PathBuf::from(format!("/usr/local/bin/{}", name));

    if alternatives.contains_key(name) {
        let ref mut alts = alternatives.get_mut(name).unwrap();
        let ref mut links = alts.alternatives;

        let position = match links.iter()
                                  .map(|e| &e.target)
                                  .position(|t| t == &target_path) {
            Some(i) => i,
            None => {
                links.push(Alternative{
                    name: String::from(name),
                    target: target_path,
                    location,
                    priority: weight
                });

                links.len() - 1
            },
        };
    }
}

fn commit_alternatives(alternatives: &AlternativeMap) {
    for (name, links) in alternatives.iter() {
        write_links(name, &links);
        remove_old_links(name);
        rename_new_links(name);
    }
}

fn write_links(name: &str, links: &Alternatives) {
    let filename = format!("/etc/alternatives/{}.json.tmp", name);
    let mut temp = match std::fs::File::create(&filename) {
        Ok(f) => f,
        Err(e) => panic!("update-alternatives: unable to create temporary \
                         file: {}", e.description()),
    };

    let serialized = match serde_json::to_string(links) {
        Ok(s) => s,
        Err(e) => panic!("update-alternatives: error while serializing \
                         alternatives for {}: {}", name, e.description()),
    };

    match write!(temp, "{}", serialized) {
        Err(e) => panic!("update-alternatives: error while writing to file \
                         {}: {}", filename, e.description()),
        _ => (),
    }
}

fn remove_old_links(name: &str) {
    let filename = format!("/etc/alternatives/{}.json", name);
    match std::fs::remove_file(&filename) {
        Err(e) => panic!("update-alternatives: error removing file {}: {}",
                         filename, e.description()),
        _ => (),
    }
}

fn rename_new_links(name: &str) {
    let temp_filename = format!("/etc/alternatives/{}.json.tmp", name);
    let new_filename = format!("/etc/alternatives/{}.json", name);
    
    std::fs::rename(temp_filename, new_filename).unwrap();
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
