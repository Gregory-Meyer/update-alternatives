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

#[derive(Serialize, Deserialize)]
pub struct AlternativeList {
    path: std::path::PathBuf,
    links: Vec<Alternative>,
}

impl AlternativeList {
    pub fn new<P: std::convert::AsRef<std::path::Path>>(path: P)
        -> AlternativeList {
        AlternativeList{ path: std::path::PathBuf::from(path.as_ref()),
                         links: Vec::new() }
    }

    pub fn num_links(&self) -> usize {
        self.links.len()
    }

    pub fn make_symlink(&self) -> std::io::Result<bool> {
        let (target, priority) = match self.links
                                           .iter()
                                           .max_by_key(|l| l.priority()) {
            Some(l) => (l.target(), l.priority()),
            None => return Ok(false),
        };

        if self.path.exists() {
            if self.path.is_file() {
                if let Ok(p) = self.path.read_link() {
                    if p == target {
                        return Ok(false);
                    }
                }

                if let Err(e) = std::fs::remove_file(&self.path) {
                    return Err(e);
                }
            } else if self.path.is_dir() {
                if let Err(e) = std::fs::remove_dir(&self.path) {
                    return Err(e);
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists, "path already exists"
                ));
            }
        }

        if let Err(e) = std::os::unix::fs::symlink(target, &self.path) {
            return Err(e);
        }

        println!("update-alternatives: created symlink from {} to {} with \
                 priority {}", self.path.display(), target.display(), priority);
        Ok(true)
    }

    pub fn add_alternative(&mut self, to_add: Alternative) -> bool {
        let target = to_add.target().to_path_buf();

        match self.links.iter().position(|a| a.target() == target) {
            Some(i) => {
                if self.links[i].priority() == to_add.priority() {
                    return false;
                }
                
                self.links[i] = to_add;

                true
            }
            None => {
                self.links.push(to_add);

                true
            }
        }
    }

    pub fn remove_alternative<P: std::convert::AsRef<std::path::Path>>(
        &mut self, target: P
    ) -> bool {
        let target_path = target.as_ref();

        if let Some(p) = self.links
                             .iter()
                             .position(|a| a.target() == target_path) {
            self.links.remove(p);

            return true;
        }

        false
    }
}

impl std::fmt::Display for AlternativeList {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Err(e) = writeln!(formatter, "alternatives for {}:",
                                 self.path.display()) {
            return Err(e);
        }

        for alternative in self.links.iter() {
            if let Err(e) = writeln!(formatter, "    {}", alternative) {
                return Err(e);
            }
        }

        Ok(())
    }
}
