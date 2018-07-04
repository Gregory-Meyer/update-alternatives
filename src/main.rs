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
        .version("0.3.1")
        .author("Gregory Meyer <gregjm@umich.edu>")
        .about("handles symlinking for multiple files")
        .long_about("places symlinks it creates in /usr/local/bin/$NAME, where \
                    NAME is the specified name of the link")
        .subcommand(clap::SubCommand::with_name("list")
                        .about("list alternatives for a given name")
                        .arg(clap::Arg::with_name("NAME")
                                 .help("the name to query")
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
