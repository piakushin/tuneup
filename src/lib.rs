#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug, PartialEq)]
pub enum Error {
    SerializationFailed(String),
    ConfigDoesNotEsixt,
    DeserializationFailed(String),
    FileOpenFailed(String),
    FileDoesNotSet,
}

struct Config {
    parameters: HashMap<String, String>,
    file: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            file: None,
        }
    }

    fn file(mut self, path: &str) -> Self {
        self.file = Some(path.to_owned());
        self
    }

    fn read_from_file(&mut self) -> Result<(), Error> {
        if let Some(s) = &self.file {
            match File::open(s) {
                Ok(f) => match serde_yaml::from_reader(f) {
                    Ok(m) => {
                        self.parameters = m;
                        Ok(())
                    }
                    Err(e) => Err(Error::SerializationFailed(e.to_string())),
                },
                Err(e) => Err(Error::FileOpenFailed(e.to_string())),
            }
        } else {
            Err(Error::FileDoesNotSet)
        }
    }

    fn add<T>(&mut self, name: &str, value: T) -> Result<(), Error>
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        match serde_yaml::to_string(&(name, value)) {
            Ok(s) => {
                self.parameters.insert(name.to_owned(), s);
                Ok(())
            }
            Err(e) => Err(Error::SerializationFailed(e.to_string())),
        }
    }

    fn get<T>(&mut self, name: &str) -> Result<T, Error>
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        if let Some(s) = self.parameters.get(name) {
            match serde_yaml::from_str::<(String, T)>(s) {
                Ok((_, v)) => Ok(v),
                Err(e) => Err(Error::DeserializationFailed(e.to_string())),
            }
        } else {
            Err(Error::ConfigDoesNotEsixt)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

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
        println!("{:?}", temp);

        let temp: Test1 = c.get("test1").unwrap();
        println!("{:?}", temp);
    }

    #[test]
    fn read_from_unset_file() {
        let mut c = Config::new();
        assert_eq!(c.read_from_file().unwrap_err(), ConfigError::FileDoesNotSet);
    }

    #[test]
    fn open_file_not_exist() {
        let test_file = "config-test.yaml";
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
        let mut c = Config::new().file(test_file);
        assert!(c.read_from_file().is_err());
    }

    #[test]
    fn read_from_file_serialization_failed() {
        let test_file = "config-test.yaml";
        let mut f = fs::File::create(test_file).unwrap();
        let test_input = "1234567";
        f.write_all(test_input.as_bytes()).unwrap();
        f.flush().unwrap();
        let mut c = Config::new().file(test_file);
        assert!(c.read_from_file().is_err());
        if let Err(e) = fs::remove_file(test_file) {
            println!("{:#?}", e);
        }
    }

    #[test]
    fn deserialize_from_file() {
        let test_file = "config-test.yaml";
        let mut f = fs::File::create(test_file).unwrap();
        let test_input = "test2\n  field: 123";
        f.write_all(test_input.as_bytes()).unwrap();
        f.flush().unwrap();
        let mut c = Config::new().file(test_file);
        c.read_from_file().unwrap();
        assert!(c.get::<Test2>("test1").is_err());
        assert!(c.get::<Test2>("test2").is_ok());
        let temp: Test2 = c.get("test2").unwrap();
        println!("{:?}", temp);
        assert_eq!(temp.field, 123);

        let temp: Test1 = c.get("test1").unwrap();
        println!("{:?}", temp);
    }
}
