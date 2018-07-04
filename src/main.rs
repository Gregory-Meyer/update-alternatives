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
            println!("alternatives for {}:", link);

            for alternative in alternatives[link].alternatives.iter() {
                println!("    {}", alternative);
            }
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
        mutated = true;
    } else {
        println!("{}", matches.usage());
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

        let name = path.file_stem().unwrap(); // should have a name
        let key = String::from(name.to_string_lossy());

        let parsed = parse_alternatives_file(&path);

        println!("found link {} with {} alternatives",
                 name.to_string_lossy(), parsed.alternatives.len());

        alternatives.insert(key, parsed);
    }

    return alternatives;
}

// path must be a normal file
fn parse_alternatives_file(path: &std::path::Path) -> Alternatives {
    let name = String::from(path.file_stem().unwrap().to_string_lossy());

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
        Ok(a) => a,
        Err(e) => panic!("update-alternatives: unable to parse file {}: {}",
                         path.to_string_lossy(), e.description()),
    }
}

fn add_alternative(alternatives: &mut AlternativeMap, target: &str,
                   name: &str, weight: i32) {
    let target_path = std::path::PathBuf::from(target);
    let location = format!("/usr/local/bin/{}", name);
    let location_path = std::path::PathBuf::from(&location);

    if alternatives.contains_key(name) {
        let ref mut alts = alternatives.get_mut(name).unwrap();

        if alts.location != location_path {
            println!("updated location from {} to {}",
                     alts.location.to_string_lossy(), location);

            alts.location = location_path;
        }

        let ref mut links = alts.alternatives;

        match links.iter().map(|e| &e.target).position(|t| t == &target_path) {
            Some(i) => {
                let link = &mut links[i];
                link.priority = weight;

                println!("updated priority of {} to {}", target, weight);
            },
            None => {
                links.push(Alternative{
                    target: target_path,
                    priority: weight
                });

                println!("added alternative {} with priority {}",
                         target, weight);
            },
        }
    } else {
        alternatives.insert(
            String::from(name),
            Alternatives{
                location: location_path,
                alternatives: vec![Alternative{
                    target: target_path,
                    priority: weight
                }]
            }
        );
    }
}

fn commit_alternatives(alternatives: &AlternativeMap) {
    for (name, links) in alternatives.iter() {
        write_links(&name, &links);
        remove_old_links(&name);
        rename_new_links(&name);
        set_symlink(&links);
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

    match writeln!(temp, "{}", serialized) {
        Err(e) => panic!("update-alternatives: error while writing to file \
                         {}: {}", filename, e.description()),
        _ => (),
    }
}

fn remove_old_links(name: &str) {
    let filename = format!("/etc/alternatives/{}.json", name);
    let path = std::path::Path::new(&filename);

    if !path.exists() {
        return;
    }

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

fn set_symlink(alternatives: &Alternatives) {
    if alternatives.alternatives.len() == 0 {
        return;
    }

    let location = &alternatives.location;
    let max = alternatives.alternatives
                          .iter()
                          .max_by_key(|l| l.priority)
                          .unwrap();
    let target = &max.target;

    if location.exists() {
        match std::fs::remove_file(&location) {
            Err(e) => panic!("update-alternatives: error removing file {}: {}",
                             location.to_string_lossy(), e.description()),
            _ => (),
        }
    }

    match std::os::unix::fs::symlink(target, location) {
        Err(e) => panic!("update-alternatives: could not create symlink from \
                         {} to {}: {}", location.to_string_lossy(),
                         target.to_string_lossy(), e.description()),
        _ => (),
    }

    println!("using {} with priority {}", target.to_string_lossy(),
             max.priority);
}

type AlternativeMap = std::collections::HashMap<String, Alternatives>;

#[derive(Serialize, Deserialize)]
struct Alternative {
    target: std::path::PathBuf,
    priority: i32,
}

impl std::fmt::Display for Alternative {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.target.to_string_lossy(),
               self.priority)
    }
}

#[derive(Serialize, Deserialize)]
struct Alternatives {
    location: std::path::PathBuf,
    alternatives: Vec<Alternative>,
}
