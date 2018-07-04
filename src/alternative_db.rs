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

extern crate std;

extern crate serde_json;

use super::alternative::Alternative;
use super::alternative_list::AlternativeList;
use super::filesystem;

type AlternativeTable = std::collections::HashMap<String, AlternativeList>;

pub struct AlternativeDb {
    table: AlternativeTable,
}

impl AlternativeDb {
    pub fn from_folder<P: std::convert::AsRef<std::path::Path>>(folder: P)
        -> std::io::Result<AlternativeDb> {
        let folder_path = folder.as_ref();
        let children = match folder_path.read_dir() {
            Ok(c) => c,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(AlternativeDb{ table: AlternativeTable::new() });
                }

                return Err(e);
            },
        };

        let to_reserve = estimate_size(&children);
        let mut table = AlternativeTable::with_capacity(to_reserve);

        for child in children {
            let entry = match child {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("update-alternatives: unable to read entry of \
                              directory {}: {}", folder_path.display(), e);

                    continue;
                },
            };

            let path = entry.path();

            let name = String::from(match path.file_stem() {
                Some(s) => s.to_string_lossy(),
                None => {
                    println!("update-alternatives: skipping entry {}...",
                             path.display());

                    continue;
                },
            });

            let contents = match filesystem::read_file(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("update-alternatives: could not read file {}: {}",
                              path.display(), e);

                    continue;
                }
            };

            let list: AlternativeList = match serde_json::from_str(&contents) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("update-alternatives: unable to \
                             deserialize {}: {}", path.display(), e);

                    continue;
                }
            };

            println!("update-alternatives: loading alternative for {} with {} \
                     entries...", name, list.num_links());
            table.insert(name, list);
        }

        Ok(AlternativeDb{ table })
    }

    pub fn num_alternatives(&self) -> usize {
        self.table.len()
    }

    pub fn has_alternatives(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn alternatives(&self, name: &str) -> Option<&AlternativeList> {
        if !self.has_alternatives(name) {
            return None;
        }

        Some(&self.table[name])
    }

    pub fn add_alternative(&mut self, name: &str,
                           to_add: Alternative) -> bool {
        if !self.has_alternatives(&name) {
            let path = format!("/usr/local/bin/{}", name);

            self.table.insert(name.to_string(), AlternativeList::new(path));
        }

        let list = self.table.get_mut(name).unwrap();

        list.add_alternative(to_add)
    }

    pub fn remove_alternative<P: std::convert::AsRef<std::path::Path>>(
        &mut self, name: &str, target: P
    ) -> bool {
        if !self.has_alternatives(name) {
            return false;
        }

        let list = self.table.get_mut(name).unwrap();

        list.remove_alternative(target)
    }

    pub fn write_out<P: std::convert::AsRef<std::path::Path>>(&self, folder: P)
        -> std::io::Result<usize> {
        let folder_path = folder.as_ref();

        if !folder_path.exists() {
            if let Err(e) = std::fs::create_dir_all(folder_path) {
                return Err(e);
            }
        } else if !folder_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists, "path is not a directory"
            ));
        }

        let mut written: usize = 0;

        for (name, list) in self.table.iter() {
            let db_file = folder_path.join(name).with_extension("json");

            if let Err(e) = AlternativeDb::rename_existing(&db_file) {
                eprintln!("update-alternatives: could not rename file {}: {}",
                          db_file.display(), e);
                continue;
            }

            match AlternativeDb::write_list(list, &db_file) {
                Ok(n) => {
                    written += n;

                    AlternativeDb::cleanup(&db_file);
                },
                Err(e) => {
                    if let Err(e) = AlternativeDb::recover(&db_file) {
                        return Err(e);
                    }

                    return Err(e);
                },
            }
        }

        Ok(written)
    }

    pub fn write_links(&self) -> std::io::Result<()> {
        for list in self.table.values() {
            if let Err(e) = list.make_symlink() {
                return Err(e);
            }
        }

        Ok(())
    }

    fn rename_existing(link: &std::path::Path) -> std::io::Result<()> {
        if !link.exists() {
            return Ok(());
        }

        let new_link = link.with_extension("json.old");

        std::fs::rename(link, &new_link)
    }

    fn write_list(list: &AlternativeList,
                  path: &std::path::Path) -> std::io::Result<usize> {
        let mut file = match std::fs::File::create(path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let to_write = match serde_json::to_string(list) {
            Ok(s) => s,
            Err(e) => return Err(std::io::Error::from(e)),
        };

        if let Err(e) = file.write_all(to_write.as_bytes()) {
            Err(e)
        } else {
            Ok(to_write.len())
        }
    }

    fn cleanup(link: &std::path::Path) {
        if let Err(e) = AlternativeDb::remove_renamed(&link) {
            eprintln!("update-alternatives: could not remove {}.old: {}",
                      link.display(), e);
        }
    }

    fn recover(link: &std::path::Path) -> std::io::Result<()> {
        if let Err(e) = AlternativeDb::recover_backup(link) {
            eprintln!("update-alternatives: could not recover {}.old: {}",
                      link.display(), e);

            return Err(e);
        }

        Ok(())
    }

    fn remove_renamed(link: &std::path::Path) -> std::io::Result<()> {
        let to_remove = link.with_extension("json.old");

        if !to_remove.exists() {
            return Ok(());
        }

        std::fs::remove_file(to_remove)
    }

    fn recover_backup(link: &std::path::Path) -> std::io::Result<()> {
        let renamed = link.with_extension("json");

        std::fs::rename(&renamed, link)
    }
}

fn estimate_size<I: std::iter::Iterator>(iter: &I) -> usize {
    let (lower_bound, upper_bound) = iter.size_hint();

    match upper_bound {
        Some(v) => v,
        None => lower_bound,
    }
}
