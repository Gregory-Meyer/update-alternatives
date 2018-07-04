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
    let matches = clap::App::new("update-alternatives")
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
        .get_matches();

    let mut db = match AlternativeDb::from_folder("/etc/alternatives") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("update-alternatives: could not read folder \
                      /etc/alternatives: {}", e);

            std::process::exit(1);
        }
    };

    println!("update-alternatives: parsed {} alternatives",
             db.num_alternatives());

    let mutated;

    if let Some(list_matches) = matches.subcommand_matches("list") {
        let name = list_matches.value_of("NAME").unwrap();

        match db.alternatives(name) {
            Some(alternatives) => {
                print!("update-alternatives: {}", alternatives);
            },
            None => {
                eprintln!("update-alternatives: no alternatives found for {}",
                          name);
            }
        }

        mutated = false;
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        let target = add_matches.value_of("TARGET").unwrap();
        let name = add_matches.value_of("NAME").unwrap();
        let weight_str = add_matches.value_of("WEIGHT").unwrap();

        let weight: i32 = match weight_str.parse() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("update-alternatives: could not parse {} as \
                          weight: {}", weight_str, e);

                std::process::exit(1);
            },
        };

        mutated = db.add_alternative(name, Alternative::from_parts(target,
                                                                   weight));

        if mutated {
            println!("update-alternatives: added alternative {} for {} with \
                     priority {}", target, name, weight);
        }
    } else if let Some(remove_matches) = matches.subcommand_matches("remove") {
        let target = remove_matches.value_of("TARGET").unwrap();
        let name = remove_matches.value_of("NAME").unwrap();

        mutated = db.remove_alternative(name, target);

        if mutated {
            println!("update-alternatives: removed alternative {} for {}",
                     target, name);
        }
    } else {
        mutated = false;
    }

    if mutated {
        if let Err(e) = db.write_out("/etc/alternatives") {
            eprintln!("update-alternatives: could not commit changes to \
                      /etc/alternatives: {}", e);

            std::process::exit(1);
        } else if let Err(e) = db.write_links() {
            eprintln!("update-alternatives: could not write symlinks: {}", e);

            std::process::exit(1);
        }
    }
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
