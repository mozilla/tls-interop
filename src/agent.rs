extern crate mio;
extern crate mio_extras;
extern crate rustc_serialize;

use config::*;
use mio::tcp::{TcpListener, TcpStream};
use mio::*;
use mio_extras::channel;
use mio_extras::channel::Receiver;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::Duration;

const SERVER: Token = mio::Token(1);
const STATUS: Token = mio::Token(2);
const ERR_CIPHER_BLACKLISTED: i32 = 89;

#[allow(dead_code)]
pub struct Agent {
    pub name: String,
    path: String,
    args: Vec<String>,
    pub socket: TcpStream,
    child: Receiver<Output>,
    pub alive: bool,
    exit_value: Option<ExitStatus>,
}

impl Agent {
    pub fn new(
        name: &str,
        path: &str,
        agent: &Option<TestCaseAgent>,
        cipher_map: &CipherMap,
        args: Vec<String>,
        ipv4: bool,
    ) -> Result<Agent, i32> {
        // IPv6 listener by default, IPv4 fallback, unless IPv4 is forced.
        let addr6 = "[::1]:0".parse().unwrap();
        let addr4 = "127.0.0.1:0".parse().unwrap();
        let listener = match ipv4 {
            false => TcpListener::bind(&addr6)
                .or_else(|_| TcpListener::bind(&addr4))
                .unwrap(),
            true => TcpListener::bind(&addr4).unwrap(),
        };

        let ossl_cipher_format = path.contains("bssl_shim") || path.contains("ossl_shim");
        // Start the subprocess.
        let mut command = Command::new(path.to_owned());

        // Process parameters.
        if let Some(ref a) = *agent {
            if let Some(ref min) = a.min_version {
                command.arg("-min-version");
                command.arg(min.to_string());
            }
            if let Some(ref min) = a.max_version {
                command.arg("-max-version");
                command.arg(min.to_string());
            }
            if let Some(ref cipher) = a.cipher {
                if cipher_map.check_blacklist(cipher, path) {
                    return Err(ERR_CIPHER_BLACKLISTED);
                }
                match ossl_cipher_format {
                    true => command.arg("-cipher").arg(
                        cipher_map.name_to_ossl(cipher)
                    ),
                    false => command.arg("-nss-cipher").arg(
                        cipher
                    ),
                };
            }
            if let Some(ref flags) = a.flags {
                for f in flags {
                    command.arg(f);
                }
            }
        }

        // Add specific args.
        // Modify cipher arguments to match the format required by the different shims.
        let mut cipher_arg = false;
        for a in &args {
            let mut arg = a.clone();
            if cipher_arg {
                if cipher_map.check_blacklist(&arg, path) {
                    return Err(ERR_CIPHER_BLACKLISTED);
                }
                match ossl_cipher_format {
                    true => command.arg(
                        cipher_map.name_to_ossl(&arg),
                    ),
                    false => command.arg(
                        arg
                    ),
                };
                cipher_arg = false;
                continue;
            }
            if arg.contains("-cipher") {
                if !ossl_cipher_format {
                    arg.insert_str(0, "-nss")
                }
                cipher_arg = true;
            }
            command.arg(arg);
        }

        // Add common args.
        command.arg("-port");
        command.arg(listener.local_addr().unwrap().port().to_string());
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        debug!("Executing command {:?}", &command);
        let child = command.spawn().expect("Failed spawning child process.");

        // Listen for connect
        // Create an poll instance
        let poll = Poll::new().unwrap();
        poll.register(&listener, SERVER, Ready::readable(), PollOpt::level())
            .unwrap();
        let mut events = Events::with_capacity(1024);

        // This is gross, but we can't reregister channels.
        // https://github.com/carllerche/mio/issues/506
        let (txf, rxf) = channel::channel::<Output>();
        let (txf2, rxf2) = channel::channel::<Output>();

        poll.register(&rxf, STATUS, Ready::readable(), PollOpt::level())
            .unwrap();

        thread::spawn(move || {
            let output = child
                .wait_with_output()
                .expect("Failed waiting for subprocess");

            txf.send(output.clone()).ok();
            txf2.send(output.clone()).ok();
        });

        poll.poll(&mut events, None).unwrap();
        debug!("Poll finished!");

        match events.iter().next().unwrap().token() {
            SERVER => {
                let sock = listener.accept();

                debug!("Accepted");
                Ok(Agent {
                    name: name.to_owned(),
                    path: path.to_owned(),
                    args: args,
                    socket: sock.unwrap().0,
                    child: rxf2,
                    alive: true,
                    exit_value: None,
                })
            }
            STATUS => {
                let output = rxf.try_recv().unwrap();
                info!("Failed {}", output.status);
                println!(
                    "Stderr: \n{}, \nStdout: \n{}",
                    String::from_utf8(output.stderr.clone()).unwrap(),
                    String::from_utf8(output.stdout.clone()).unwrap()
                );
                Err(output.status.code().unwrap())
            }
            _ => Err(-1),
        }
    }

    // Read the status from the subthread.
    pub fn check_status(&self) -> Output {
        debug!("Getting status for {}", self.name);
        // try_recv() is nonblocking, so poll until it's readable.
        let poll = Poll::new().unwrap();
        poll.register(&self.child, STATUS, Ready::readable(), PollOpt::level())
            .unwrap();
        let mut events = Events::with_capacity(1);
        // poll() used to cause indefinite blocking of the main thread in rare cases, and,
        // consequently, intermittent timeouts of the whole test suite. It is now set to time out
        // and stop blocking after 30 seconds.
        // The output of the subthread should always be readable on the channel by that time. It
        // seems that poll() missed the event if it was registered after the arrival of the output.
        // There's currently no proof this fixes the issue, but no timeouts have been observed
        // since the change was implemented.
        poll.poll(&mut events, Some(Duration::new(30, 0))).unwrap();
        debug!("Poll successful or timed out. Trying to receive output...");
        let output = self
            .child
            .try_recv()
            .expect("Failed to receive output from subthread.");
        let code = output.status.code().unwrap_or(-1);
        debug!("Exit status for {} = {}", self.name, code);
        output.clone()
    }
}
