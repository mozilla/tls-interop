use std::fs;
use std::io::prelude::*;
use rustc_serialize::json;
use std::collections::HashMap;

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
    pub ciphers: Option<Vec<String>>
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
pub struct CipherBlacklist {
    pub blacklist: Option<HashMap<String, Vec<String>>>,
}

impl CipherBlacklist {
    pub fn new() -> CipherBlacklist {
        CipherBlacklist {
            blacklist: None,
        }
    }
    
    pub fn init(&mut self, file: &str) {
        let mut bl = fs::File::open(file).unwrap();
        let mut bls = String::from("");
        bl.read_to_string(&mut bls)
            .expect("Could not read file to string.");
        *self = json::decode(&bls).expect("Malformed JSON blacklist file.");
    }
    
    pub fn check(&self, cipher: &str, shim: &str) -> bool {
        if let Some(list) = self.blacklist.clone() {
            if let Some(l) = list.get(cipher) {
                for s in l {
                    if shim.contains(s) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}
