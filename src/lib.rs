#![feature(inner_deref)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::fs::File;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub enum Error {
    SerializationFailed(String),
    ConfigDoesNotEsixt,
    DeserializationFailed(String),
    FileOpenFailed(String),
    FileDoesNotSet,
}

#[derive(Default)]
pub struct Config {
    root: Value,
    file: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            root: Value::Mapping(Mapping::new()),
            file: None,
        }
    }

    pub fn with_file(mut self, path: &str) -> Self {
        self.file = Some(path.to_owned());
        self
    }

    pub fn read_from_file(&mut self) -> Result<(), Error> {
        match File::open(self.file()?) {
            Ok(f) => match serde_yaml::from_reader(f) {
                Ok(m) => {
                    self.root = m;
                    Ok(())
                }
                Err(e) => Err(Error::DeserializationFailed(e.to_string())),
            },
            Err(e) => Err(Error::FileOpenFailed(e.to_string())),
        }
    }

    pub fn write_to_file(&mut self) -> Result<(), Error> {
        use std::fs::OpenOptions;
        println!("{:#?}", serde_yaml::to_string(&self.root));
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.file()?)
        {
            Ok(f) => {
                serde_yaml::to_writer(f, &self.root)
                    .map_err(|e| Error::SerializationFailed(e.to_string()))?;
            }
            Err(e) => {
                return Err(Error::FileOpenFailed(e.to_string()));
            }
        }
        Ok(())
    }

    pub fn add<T>(&mut self, name: &str, value: T) -> Result<(), Error>
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        self.root.as_mapping_mut().unwrap().insert(
            serde_yaml::to_value(name).unwrap(),
            serde_yaml::to_value(value).unwrap(),
        );
        Ok(())
    }

    pub fn get<T>(&mut self, name: &str) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        if let Some(s) = self.root.get(name) {
            match serde_yaml::from_value::<T>(s.to_owned()) {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::DeserializationFailed(e.to_string())),
            }
        } else {
            Err(Error::ConfigDoesNotEsixt)
        }
    }

    fn file(&self) -> Result<&str, Error> {
        self.file.deref().ok_or(Error::FileDoesNotSet)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};

    use super::{Config, Error as ConfigError};

    #[derive(Serialize, Deserialize, Debug)]
    struct Test1;

    #[derive(Serialize, Deserialize, Debug)]
    struct Test2 {
        field: usize,
    }

    #[test]
    fn create_new() {
        let mut c = Config::new();

        c.add("test1", Test1 {}).unwrap();
        c.add("test2", Test2 { field: 0 }).unwrap();

        let temp: Test2 = c.get("test2").unwrap();
        println!("{:#?}", temp);

        let temp: Test1 = c.get("test1").unwrap();
        println!("{:#?}", temp);
    }

    #[test]
    fn read_from_unset_file() {
        let mut c = Config::new();
        assert_eq!(c.read_from_file().unwrap_err(), ConfigError::FileDoesNotSet);
    }

    #[test]
    fn open_file_not_exist() {
        let test_file = "config-test-ofne.yaml";
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
        let mut c = Config::new().with_file(test_file);
        assert!(c.read_from_file().is_err());
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
    }

    #[test]
    fn deserialize_from_file() {
        let test_file = "config-test-dff.yaml";
        let mut f = fs::File::create(test_file).unwrap();
        let test_input = "test2:\n  field: 123";
        f.write_all(test_input.as_bytes()).unwrap();
        f.flush().unwrap();
        let mut f_r = fs::File::open(test_file).unwrap();
        let mut fc = String::new();
        f_r.read_to_string(&mut fc).unwrap();
        println!("file content: \n{:#?}\n", fc);
        let mut c = Config::new().with_file(test_file);
        c.read_from_file().unwrap();
        assert!(c.get::<Test2>("test1").is_err());
        assert!(c.get::<Test2>("test2").is_ok());
        let temp: Test2 = c.get("test2").unwrap();
        println!("{:#?}", temp);
        assert_eq!(temp.field, 123);
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
    }

    #[test]
    fn read_from_file_add_struct_write_to_file() {
        #[derive(Serialize, Deserialize, Debug)]
        struct Test3 {
            field_usize_1: usize,
            field_string_1: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        struct Test4 {
            field_usize_1: usize,
            field_string_1: String,
            field_f64_1: f64,
        }
        let test_file = "config-test-rffaswtf.yaml";
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
        let mut f = fs::File::create(test_file).unwrap();
        let test_input = "test3:
          field_usize_1: 12
          field_string_1: qwerty
        ";
        f.write_all(test_input.as_bytes()).unwrap();
        let mut fc = String::new();
        let mut f_r = fs::File::open(test_file).unwrap();
        f_r.read_to_string(&mut fc).unwrap();
        println!("file content: >>>\n{}\n>>>", fc);
        let mut c = Config::new().with_file(test_file);
        c.read_from_file().unwrap();
        let t = Test4 {
            field_usize_1: 1234,
            field_string_1: "qwertyasdfgh".to_owned(),
            field_f64_1: 123.456,
        };
        c.add("test4", t).unwrap();
        c.write_to_file().unwrap();
        let mut output = String::new();
        f_r.read_to_string(&mut output).unwrap();
        println!("file content: <<<\n{}\n<<<", output);
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
    }
}
