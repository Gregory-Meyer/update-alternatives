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

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

mod alternative;
mod alternative_db;
mod alternative_list;
mod filesystem;

use alternative::Alternative;
use alternative_db::AlternativeDb;

fn main() {
    let matches = app().get_matches();

    let mut db = match read_db("/etc/alternatives") {
        Ok(d) => d,
        Err(_) => std::process::exit(1),
    };

    let mutated;

    if let Some(list_matches) = matches.subcommand_matches("list") {
        mutated = list(&db, list_matches);
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        mutated = add(&mut db, add_matches);
    } else if let Some(remove_matches) = matches.subcommand_matches("remove") {
        mutated = remove(&mut db, remove_matches);
    } else {
        mutated = false;
    }

    if mutated && commit(&db).is_err() {
        std::process::exit(1);
    }
}

fn read_db<P: std::convert::AsRef<std::path::Path>>(path: P)
-> std::io::Result<AlternativeDb> {
    match AlternativeDb::from_folder(path) {
        Ok(d) => {
            println!("update-alternatives: parsed {} alternatives",
                     d.num_alternatives());

            Ok(d)
        },
        Err(e) => {
            eprintln!("update-alternatives: could not read folder \
                      /etc/alternatives: {}", e);

            Err(e)
        }
    }
}

fn list(db: &AlternativeDb, matches: &clap::ArgMatches) -> bool {
    let name = matches.value_of("NAME").unwrap();

    match db.alternatives(name) {
        Some(alternatives) => {
            print!("update-alternatives: {}", alternatives);
        },
        None => {
            eprintln!("update-alternatives: no alternatives found for {}", name);
        }
    }

    false
}

fn add(db: &mut AlternativeDb, matches: &clap::ArgMatches) -> bool {
    let target = matches.value_of("TARGET").unwrap();
    let name = matches.value_of("NAME").unwrap();
    let weight_str = matches.value_of("WEIGHT").unwrap();

    let weight: i32 = match weight_str.parse() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("update-alternatives: could not parse {} as \
                      weight: {}", weight_str, e);

            std::process::exit(1);
        },
    };

    if db.add_alternative(name, Alternative::from_parts(target, weight)) {
        println!("update-alternatives: added alternative {} for {} with \
                 priority {}", target, name, weight);

        return true;
    }

    false
}

fn remove(db: &mut AlternativeDb, matches: &clap::ArgMatches) -> bool {
    let target = matches.value_of("TARGET").unwrap();
    let name = matches.value_of("NAME").unwrap();

    if db.remove_alternative(name, target) {
        println!("update-alternatives: removed alternative {} for {}",
                 target, name);

        return true;
    }

    false
}

fn commit(db: &AlternativeDb) -> std::io::Result<()> {
    if let Err(e) = db.write_out("/etc/alternatives") {
        eprintln!("update-alternatives: could not commit changes to \
                  /etc/alternatives: {}", e);

        Err(e)
    } else if let Err(e) = db.write_links() {
        eprintln!("update-alternatives: could not write symlinks: {}", e);

        Err(e)
    } else {
        Ok(())
    }
}

fn app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("update-alternatives")
        .version(crate_version!())
        .author("Gregory Meyer <gregjm@umich.edu>")
        .about(ABOUT)
        .subcommand(clap::SubCommand::with_name("list")
                        .about(LIST_ABOUT)
                        .arg(clap::Arg::with_name("NAME")
                                 .help("The name of the alternatives to query")
                                 .value_name("NAME")
                                 .short("n")
                                 .long("name")
                                 .required(true)
                                 .takes_value(true)))
        .subcommand(clap::SubCommand::with_name("add")
                        .about(ADD_ABOUT)
                        .arg(clap::Arg::with_name("TARGET")
                                 .help("The target of the alternative to add")
                                 .value_name("TARGET")
                                 .short("t")
                                 .long("target")
                                 .required(true)
                                 .takes_value(true))
                        .arg(clap::Arg::with_name("NAME")
                                 .help("The name of the alternative to add")
                                 .value_name("NAME")
                                 .short("n")
                                 .long("name")
                                 .required(true)
                                 .takes_value(true))
                        .arg(clap::Arg::with_name("WEIGHT")
                                 .help("The priority of the alternative to add")
                                 .value_name("WEIGHT")
                                 .short("w")
                                 .long("weight")
                                 .required(true)
                                 .takes_value(true)))
        .subcommand(clap::SubCommand::with_name("remove")
                        .about(REMOVE_ABOUT)
                        .arg(clap::Arg::with_name("TARGET")
                                 .help("The target of the \
                                       alternative to remove")
                                 .value_name("TARGET")
                                 .short("t")
                                 .long("target")
                                 .required(true)
                                 .takes_value(true))
                        .arg(clap::Arg::with_name("NAME")
                                 .help("The name of the alternative to remove")
                                 .value_name("NAME")
                                 .short("n")
                                 .long("name")
                                 .required(true)
                                 .takes_value(true)))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .setting(clap::AppSettings::GlobalVersion)
}

static ABOUT: &'static str =
    "Manages symlinks to be placed in /usr/local/bin. Data is stored in \
    /etc/alternatives for persistence between invocations. Provides similar \
    functionality to Debian's update-alternatives, but with a slightly \
    different interface. Alternatives are selected by comparing their assigned \
    priority values, with the highest priority being linked to.";

static LIST_ABOUT: &'static str =
    "Lists all alternatives for <NAME> and their assigned priority.";

static ADD_ABOUT: &'static str =
    "Adds or modifies an alternative for <NAME> that points to <TARGET> with \
    priority <WEIGHT>. If the database is modified, requires read/write access \
    to /etc/alternatives and /usr/local/bin.";

static REMOVE_ABOUT: &'static str =
    "If one exists, removes the alternative for <NAME> that points to \
    <TARGET>. If the database is modified, requires read/write access to \
    /etc/alternatives and /usr/local/bin.";
