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

use std::io::{Read, Write};

pub fn remove<P: std::convert::AsRef<std::path::Path>>(path: P)
-> std::io::Result<()> {
    let concrete: &std::path::Path = path.as_ref();

    if !concrete.exists() {
        Ok(())
    } else if concrete.is_dir() {
        std::fs::remove_dir_all(concrete)
    } else if concrete.is_file() {
        std::fs::remove_file(concrete)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "path is neither a directory nor a file"
        ))
    }
}

#[cfg(unix)]
pub fn symlink<P: std::convert::AsRef<std::path::Path>,
               Q: std::convert::AsRef<std::path::Path>>(
    source: P, destination: Q
) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
pub fn symlink<P: std::convert::AsRef<std::path::Path>,
               Q: std::convert::AsRef<std::path::Path>>(
    source: P, destination: Q
) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

pub fn read<P: std::convert::AsRef<std::path::Path>>(path: P)
-> std::io::Result<String> {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let mut buffer = String::new();

    match file.read_to_string(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e),
    }
}

pub fn create_dir<P: std::convert::AsRef<std::path::Path>>(path: P)
-> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn write<S, P: std::convert::AsRef<std::path::Path>>(contents: S, path: P)
-> std::io::Result<usize> where String: std::convert::From<S> {
    let to_write = String::from(contents);
    let len = to_write.len();

    let mut file = match std::fs::File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    match file.write_all(to_write.as_bytes()) {
        Ok(_) => Ok(len),
        Err(e) => Err(e),
    }
}
