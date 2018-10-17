use rustc_serialize::json;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;

// The config file that corresponds to a test.
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct TestCaseAgent {
    pub min_version: Option<u32>,
    pub max_version: Option<u32>,
    pub cipher: Option<String>,
    pub flags: Option<Vec<String>>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
// These are parameters which let us run parametrized tests.
pub struct TestCaseParams {
    pub versions: Option<Vec<i32>>,
    pub ciphers: Option<Vec<String>>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct TestCase {
    pub name: String,
    pub server_key: Option<String>,
    pub client_params: Option<TestCaseParams>,
    pub server_params: Option<TestCaseParams>,
    pub shared_params: Option<TestCaseParams>,
    pub client: Option<TestCaseAgent>,
    pub server: Option<TestCaseAgent>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct TestCases {
    pub cases: Vec<TestCase>,
}


#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct CipherMapItem {
    pub comment: String,
    pub ossl_name: String,
    pub blacklist: Vec<String>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct CipherMap {
    pub map: Option<HashMap<String, CipherMapItem>>,
}

impl CipherMap {
    pub fn new() -> CipherMap {
        CipherMap { map: None }
    }

    pub fn init(&mut self, file: &str) {
        let mut f = fs::File::open(file).unwrap();
        let mut map = String::from("");
        f.read_to_string(&mut map)
            .expect("Could not read file to string.");
        *self = json::decode(&map).expect("Malformed JSON CipherMap file.");
    }

    pub fn check_blacklist(&self, cipher: &str, shim: &str) -> bool {
        if let Some(ref list) = self.map {
            if let Some(l) = list.get(cipher) {
                for s in l.blacklist.clone() {
                    if shim.contains(&s) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    pub fn name_to_ossl(&self, cipher: &str) -> String {
        if let Some(ref list) = self.map {
            if let Some(l) = list.get(cipher) {
                return l.ossl_name.clone()
            }
        }
        return String::from("");
    }
}
