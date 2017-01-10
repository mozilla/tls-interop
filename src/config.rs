// The config file that corresponds to a test.
#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
pub struct TestCaseAgent {
    pub min_version: Option<u32>,
    pub max_version: Option<u32>,
    pub flags: Option<Vec<String>>,
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
// These are parameters which let us run parametrized tests.
pub struct TestCaseParams {
    pub versions: Option<Vec<i32>>,
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub server_key: Option<String>,
    pub client_params: Option<TestCaseParams>,
    pub server_params: Option<TestCaseParams>,
    pub client: Option<TestCaseAgent>,
    pub server: Option<TestCaseAgent>,
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
pub struct TestCases {
    pub cases: Vec<TestCase>,
}
