/*
 * Copyright [2022] [Kevin Velasco]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufWriter;
use std::path::Path;

pub fn write_html_files_to_directory(
    directory: &Path,
    write: impl FnOnce(&mut BufWriter<&mut File>) -> io::Result<()>,
) -> std::io::Result<()> {
    let html_file = directory.join("index.html");
    let data_json = directory.join("data.json");

    use std::io::Write;

    let mut html_file_handle = OpenOptions::new()
        .create(true)
        .write(true)
        .open(html_file)?;
    let mut data_file_handle = OpenOptions::new()
        .create(true)
        .write(true)
        .open(data_json)?;
    html_file_handle.set_len(0)?;
    data_file_handle.set_len(0)?;

    write!(
        &mut html_file_handle,
        "{}",
        include_str!("../templates/index.html")
    )?;
    let mut writer = BufWriter::new(&mut data_file_handle);
    write(&mut writer)?;

    Ok(())
}
