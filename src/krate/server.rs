//! This module provides `Server`, a helper struct that implements a
//! syncronous client/server logic.

use crate::krate::Krate;
use crate::prelude::*;
use quote::quote;
use std::io::{BufRead, BufReader, BufWriter};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};

/// `declare!(Name, <tokens>)` defines a struct `Name` that implement
/// `quote::ToTokens`: when converted into a token stream, `Name`
/// expands to `quote!{<tokens>}`.
macro_rules! declare {
    ($name:ident, $($tt:tt)*) => {
        $($tt)*
        struct $name;
        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote!{$($tt)*})
            }
        }
    }
}
pub(crate) use declare;

pub struct Server {
    process: Child,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
    stdin: BufWriter<ChildStdin>,
    krate: Krate,
}

impl Server {
    /// Creates a `Server` out of a block of code `body` that can
    /// consume a `request` free variable. `body` should be of a type
    /// serializable by `serde`. The type of `request` needs to be
    /// constrainted in `body`: it can be anything that is
    /// deserializable by `serde`.
    pub fn from_json_fn(
        body: impl quote::ToTokens,
        deps: &HashMap<String, DependencySpec>,
    ) -> Self {
        Self::from_string_fn(
            quote! {
                let request = serde_json::from_str(&request).unwrap();
                let response = { #body };
                serde_json::to_string(&response).unwrap()
            },
            deps,
        )
    }
    /// Creates a `Server` out of a block of code `body` that can
    /// consume a `request` free variable of type `String`. The type
    /// of `body` should be `String`.
    pub fn from_string_fn(
        body: impl quote::ToTokens,
        deps: &HashMap<String, DependencySpec>,
    ) -> Self {
        let program = quote! {
            fn main() {
                use std::io::*;
                let mut reader = BufReader::new(stdin());
                loop {
                    let mut s = String::new();
                    if reader.read_line(&mut s).is_ok() {
                        let request = s.trim();
                        let anwser = { #body };
                        println!("{}", anwser);
                    } else {
                        eprintln!("Server: could not read line");
                    }
                }
            }
        };
        Self::new(&format!("{}", program), deps)
    }
    /// Creates a server out of a Rust module `source`, which is
    /// expected to implement a `main` function.
    pub fn new(source: &str, deps: &HashMap<String, DependencySpec>) -> Self {
        let mut krate = Krate::new();
        krate.add_dependencies(deps);
        krate.source(source);
        krate.use_serde();
        let mut process = krate.run();
        let stdout = BufReader::new(process.stdout.take().unwrap());
        let stderr = BufReader::new(process.stderr.take().unwrap());
        let stdin = BufWriter::new(process.stdin.take().unwrap());
        Self {
            process,
            krate,
            stdout,
            stdin,
            stderr,
        }
    }
    /// Sends a request to the server and blocks until the server
    /// returns a response.
    pub fn request(&mut self, req: &str) -> String {
        use std::io::Write;
        self.stdin.write_all(req.as_bytes()).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        self.stdin.flush().unwrap();
        let mut response = String::new();
        self.stdout.read_line(&mut response).unwrap();
        response.trim().into()
    }
    /// Similar to `request`, but with JSON values.
    pub fn request_json<T: serde::Serialize, U: serde::de::DeserializeOwned>(
        &mut self,
        req: &T,
    ) -> U {
        let request = serde_json::to_string(req).unwrap();
        let response = self.request(&request);
        serde_json::from_str(&response).unwrap_or_else(|err| {
            eprintln!("ERROR: `server::request_json` failed to parse a value with error `{err:?}`");
            let request = serde_json::to_string_pretty(req).unwrap();
            eprintln!("The (pretty printed) request was: <{}>", request);
            eprintln!("The response is: <{}>", response);
            let mut stderr = String::new();
            self.stderr.read_line(&mut stderr).unwrap();
            eprintln!("The stderr is: <{}>", stderr);
            panic!()
        })
    }
}

#[test]
fn server_string() {
    let mut server = Server::from_string_fn(
        quote! {
            request
        },
        "",
    );
    for i in ["A", "B", "C"] {
        assert_eq!(i, server.request(i));
    }
}

#[test]
fn server_json() {
    let mut server = Server::from_json_fn(
        quote! {
            let (x, y): (u8, u8) = request;
            x + y
        },
        "",
    );
    assert_eq!(13u16, server.request_json::<_, u16>(&(3u8, 10u8)));
    assert_eq!(42u16, server.request_json::<_, u16>(&(40u8, 2u8)));
}
