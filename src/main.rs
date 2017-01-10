extern crate clap;
#[macro_use]
extern crate log;
extern crate mio;
extern crate env_logger;
extern crate rustc_serialize;
use clap::{Arg, App};
use mio::*;
use mio::tcp::Shutdown;
use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;
use std::process::exit;
mod agent;
mod config;
mod test_result;
mod flatten;
mod tests;
use agent::Agent;
use test_result::TestResult;
use config::{TestCase, TestCases, TestCaseParams};
use flatten::flatten;

const CLIENT: Token = mio::Token(0);
const SERVER: Token = mio::Token(1);

fn copy_data(poll: &Poll, from: &mut Agent, to: &mut Agent) {
    let mut buf: [u8; 16384] = [0; 16384];
    let mut b = &mut buf[..];
    let rv = from.socket.read(b);
    let size = rv.unwrap_or_else(|e| {
        debug!("Error {} on {}", e, from.name);
        0
    });
    if size == 0 {
        debug!("End of file on {}", from.name);
        poll.deregister(&from.socket).expect("Could not deregister socket");
        to.socket.shutdown(Shutdown::Write).expect("Shutdown failed");
        from.alive = false;
        return;
    }
    debug!("Read {} from {} ", size, from.name);

    let b2 = &b[0..size];
    let rv = to.socket.write_all(b2);
    match rv {
        Err(_) => {
            panic!("write failed");
        }
        _ => {
            debug!("Write succeeded");
        }
    };
}

fn shuttle(client: &mut Agent, server: &mut Agent) {
    // Listen for connect
    // Create a poll instance
    let poll = Poll::new().unwrap();
    poll.register(&client.socket, CLIENT, Ready::readable(), PollOpt::level())
        .unwrap();
    poll.register(&server.socket, SERVER, Ready::readable(), PollOpt::level())
        .unwrap();
    let mut events = Events::with_capacity(1024);

    while client.alive || server.alive {
        debug!("Poll");

        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                CLIENT => {
                    copy_data(&poll, client, server);
                }
                SERVER => {
                    copy_data(&poll, server, client);
                }
                _ => unreachable!(),
            }
        }
    }
}

// The command line options passed to the runner.
pub struct TestConfig {
    client_shim: String,
    server_shim: String,
    rootdir: String,
}

// The results of the entire test run.
struct Results {
    ran: u32,
    succeeded: u32,
    failed: u32,
    skipped: u32,
}

impl Results {
    fn new() -> Results {
        Results {
            ran: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
        }
    }

    fn case_name(case: &TestCase, index: Option<u32>) -> String {
        let mut name = case.name.clone();

        match index {
            None => name,
            Some(x) => {
                name.push_str("/");
                name + &x.to_string()
            }
        }
    }

    fn update(&mut self, case: &TestCase, index: Option<u32>, result: TestResult) {
        self.ran += 1;

        info!("{}: {}", result.to_string(), Results::case_name(case, index));

        match result {
            TestResult::OK => self.succeeded += 1,
            TestResult::Skipped => self.skipped += 1,
            TestResult::Failed => {
                println!("FAILED: {}", Results::case_name(case, index));
                self.failed += 1
            }
        }
    }
}

fn make_params(params: &Option<TestCaseParams>) -> Vec<Vec<String>> {
    let mut mat = vec![];

    if let &Some(ref p) = params {
        if let Some(ref versions) = p.versions {
            let mut alist = vec![];
            for ver in versions {
                let mut args = vec![];

                args.push(String::from("-min-version"));
                args.push(ver.to_string());
                args.push(String::from("-max-version"));
                args.push(ver.to_string());

                alist.push(args);
            }
            mat.push(alist)
        }
    }

    flatten(&mat)
}

fn run_test_case_meta(results: &mut Results, config: &TestConfig, case: &TestCase) {
    if !case.client_params.is_some() && !case.server_params.is_some() {
        let dummy = vec![];
        run_test_case(results, config, case, None, &dummy, &dummy);
    } else {
        let client_args = make_params(&case.client_params);
        let server_args = make_params(&case.server_params);
        let mut index: u32 = 0;

        for c in &client_args {
            for s in &server_args {
                run_test_case(results, config, case, Some(index), c, s);
                index += 1;
            }
        }
    }
}

fn run_test_case(results: &mut Results,
                 config: &TestConfig,
                 case: &TestCase,
                 index: Option<u32>,
                 extra_client_args: &Vec<String>,
                 extra_server_args: &Vec<String>) {

    let r = run_test_case_inner(config, case, extra_client_args, extra_server_args);
    results.update(case, index, r);
}

fn run_test_case_inner(config: &TestConfig,
                       case: &TestCase,
                       extra_client_args: &Vec<String>,
                       extra_server_args: &Vec<String>)
                       -> TestResult {
    // Create the server args
    let mut server_args = extra_server_args.clone();
    server_args.push(String::from("-server"));
    let key_base = match case.server_key {
        None => String::from("rsa_1024"),
        Some(ref key) => key.clone(),
    };
    server_args.push(String::from("-key-file"));
    server_args.push(config.rootdir.clone() + &key_base + &String::from("_key.pem"));
    server_args.push(String::from("-cert-file"));
    server_args.push(config.rootdir.clone() + &key_base + &String::from("_cert.pem"));
    server_args.push(String::from("-write-then-read"));

    let mut server = match Agent::new("server", &config.server_shim, &case.server, server_args) {
        Ok(a) => a,
        Err(e) => {
            return TestResult::from_status(e);
        }
    };

    let client_args = extra_client_args.clone();
    let mut client = match Agent::new("client", &config.client_shim, &case.client, client_args) {
        Ok(a) => a,
        Err(e) => {
            return TestResult::from_status(e);
        }
    };

    shuttle(&mut client, &mut server);

    return TestResult::merge(client.check_status(), server.check_status());
}

fn main() {
    env_logger::init().expect("Could not init logging");

    let matches = App::new("TLS interop tests")
        .version("0.0")
        .arg(Arg::with_name("client")
            .long("client")
            .help("The shim to use as the client")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("server")
            .long("server")
            .help("The shim to use as the server")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("rootdir")
            .long("rootdir")
            .help("The path where the working files are")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("cases")
            .long("test-cases")
            .help("The test cases file to run")
            .takes_value(true)
            .required(true))
        .get_matches();

    let config = TestConfig {
        client_shim: String::from(matches.value_of("client").unwrap()),
        server_shim: String::from(matches.value_of("server").unwrap()),
        rootdir: String::from(matches.value_of("rootdir").unwrap()),
    };

    let mut f = File::open(matches.value_of("cases").unwrap()).unwrap();
    let mut s = String::from("");
    f.read_to_string(&mut s).expect("Could not read file to string");
    let cases: TestCases = json::decode(&s).unwrap();

    let mut results = Results::new();
    for c in cases.cases {
        run_test_case_meta(&mut results, &config, &c);
    }

    println!("Tests {}; Succeeded {}; Skipped {}, Failed {}",
             results.ran,
             results.succeeded,
             results.skipped,
             results.failed);

    if results.failed != 0 {
        exit(1);
    }

    exit(0);
}
