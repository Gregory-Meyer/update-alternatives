// Copyright (c) 2018, Gregory Meyer
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright
//       notice, this list of conditions and the following disclaimer in the
//       documentation and/or other materials provided with the distribution.
//     * Neither the name of the <organization> nor the
//       names of its contributors may be used to endorse or promote products
//       derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY 
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
// (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
// LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
// ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


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
        .version("0.2.1")
        .author("Gregory Meyer <gregjm@umich.edu>")
        .about("handles symlinking for multiple files")
        .long_about("places symlinks it creates in /usr/local/bin/$NAME, where \
                    NAME is the specified name of the link")
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
        .subcommand(clap::SubCommand::with_name("remove")
                        .about("remove a link")
                        .arg(clap::Arg::with_name("TARGET")
                                 .help("the target of the link to remove")
                                 .required(true)
                                 .index(1))
                        .arg(clap::Arg::with_name("NAME")
                                 .help("the name of the link to remove")
                                 .required(true)
                                 .index(2)))
        .get_matches();

    initialize_files();

    let mut alternatives = load_alternatives();
    let mutated;

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

        mutated = false;
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
    } else if let Some(remove_matches) = matches.subcommand_matches("remove") {
        let target = remove_matches.value_of("TARGET").unwrap();
        let name = remove_matches.value_of("NAME").unwrap();

        remove_alternative(&mut alternatives, target, name);
        mutated = true;
    } else {
        println!("{}", matches.usage());
        mutated = false;
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
        let alts = alternatives.get_mut(name).unwrap();

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

fn remove_alternative(alternatives: &mut AlternativeMap, target: &str,
                      name: &str) {
    let target_path = std::path::Path::new(target);

    if !alternatives.contains_key(name) {
        return;
    }

    let alts = alternatives.get_mut(name).unwrap();

    let found_index = match alts.alternatives
                                .iter()
                                .position(|l| l.target == target_path) {
        Some(i) => i,
        None => return,
    };

    alts.alternatives.remove(found_index);
    println!("removed alternative {}", target);
}

fn commit_alternatives(alternatives: &AlternativeMap) {
    for (name, links) in alternatives.iter() {
        write_links(&name, &links);
        remove_old_links(&name);
        rename_new_links(&name);
        set_symlink(&name, &links);
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

fn set_symlink(name: &str, alternatives: &Alternatives) {
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

    println!("using {} for {} with priority {}", target.to_string_lossy(), name,
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
