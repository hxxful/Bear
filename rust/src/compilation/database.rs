/*  Copyright (C) 2012-2018 by László Nagy
    This file is part of Bear.

    Bear is a tool to generate compilation database for clang tooling.

    Bear is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Bear is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::collections;
use std::fs;
use std::path;

use Result;


/// Represents a generic entry of the compilation database.
#[derive(Hash)]
pub struct Entry {
    pub directory: path::PathBuf,
    pub file: path::PathBuf,
    pub command: Vec<String>,
    pub output: Option<path::PathBuf>,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.directory == other.directory
            && self.file == other.file
            && self.command == other.command
    }
}

impl Eq for Entry {
}

type Entries = collections::HashSet<Entry>;


/// Represents the expected format of the JSON compilation database.
pub struct DatabaseFormat {
    command_as_array: bool,

    // Other attributes might be:
    // - output present or not
    // - paths are relative or absolute
}

impl DatabaseFormat {
    pub fn new() -> Self {
        DatabaseFormat {
            command_as_array: true,
        }
    }

    pub fn set_command_as_array(&mut self, value: bool) -> &mut Self {
        self.command_as_array = value;
        self
    }

    pub fn is_command_as_array(&self) -> bool {
        self.command_as_array
    }
}

/// Represents a JSON compilation database.
pub struct Database {
    path: path::PathBuf,
}

impl Database {
    pub fn new(path: &path::Path) -> Self {
        Database { path: path.to_path_buf(), }
    }

    pub fn load(&self) -> Result<Entries> {
        let generic_entries = inner::load(&self.path)?;
        let entries = generic_entries.iter()
            .map(|entry| inner::into(entry))
            .collect::<Result<Entries>>();
        // In case of error, let's be verbose which entries were problematic.
        if let Err(_) = entries {
            let errors = generic_entries.iter()
                .map(|entry| inner::into(entry))
                .filter_map(Result::err)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(errors.into())
        } else {
            entries
        }
    }

    pub fn save(&self, entries: &Entries, format: &DatabaseFormat) -> Result<()> {
        let generic_entries = entries.iter()
            .map(|entry| inner::from(entry, format))
            .collect::<Result<Vec<_>>>()?;
        inner::save(&self.path, &generic_entries)
    }
}


mod inner {
    use super::*;
    use serde_json;
    use shellwords;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum GenericEntry {
        StringEntry {
            directory: String,
            file: String,
            command: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            output: Option<String>,
        },
        ArrayEntry {
            directory: String,
            file: String,
            arguments: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            output: Option<String>,
        },
    }

    type GenericEntries = Vec<GenericEntry>;


    pub fn load(path: &path::Path) -> Result<GenericEntries> {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(path)?;
        let entries: GenericEntries = serde_json::from_reader(file)?;
        Ok(entries)
    }

    pub fn save(path: &path::Path, entries: &GenericEntries) -> Result<()> {
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        serde_json::ser::to_writer_pretty(file, entries)
            .map_err(|error| error.into())
    }

    pub fn from(entry: &Entry, format: &DatabaseFormat) -> Result<GenericEntry> {
        fn path_to_string(path: &path::Path) -> Result<String> {
            match path.to_str() {
                Some(str) => Ok(str.to_string()),
                None => Err(format!("Failed to convert to string {:?}", path).into()),
            }
        }

        let directory = path_to_string(entry.directory.as_path())?;
        let file = path_to_string(entry.file.as_path())?;
        let output = match entry.output {
            Some(ref path) => path_to_string(path).map(Option::Some),
            None => Ok(None),
        }?;
        if format.is_command_as_array() {
            Ok(GenericEntry::ArrayEntry {
                directory,
                file,
                arguments: entry.command.clone(),
                output
            })
        } else {
            Ok(GenericEntry::StringEntry {
                directory,
                file,
                command: shellwords::join(
                    entry.command
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_ref()),
                output
            })
        }
    }

    pub fn into(_entry: &GenericEntry) -> Result<Entry> {
        unimplemented!()
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_load_arguments() {
            let input =
                r#"{
                "directory": "/build/dir/path",
                "file": "/path/to/source/file.c",
                "arguments": ["cc", "-c", "/path/to/source/file.c"]
            }"#;

            let entry: GenericEntry = serde_json::from_str(input).unwrap();
            println!("{:?}", entry);
        }

        #[test]
        fn test_save_arguments() {
            let entry_one = GenericEntry::ArrayEntry {
                directory: "/build/dir/path".to_string(),
                file: "/path/to/source.c".to_string(),
                arguments: vec!["cc".to_string(), "-c".to_string()],
                output: None
            };
            let entry_two = GenericEntry::StringEntry {
                directory: "/build/dir/path".to_string(),
                file: "/path/to/source.c".to_string(),
                command: "cc -c /path/to/source.c -o /build/dir/path/source.o".to_string(),
                output: Some("/build/dir/path/source.o".to_string())
            };
            let inputs = vec![entry_one, entry_two];

            let output = serde_json::to_string(&inputs).unwrap();
            println!("{}", output);
        }
    }
}
