//! Prototype P2: string-keyed tokens versus capability values, in Rust.
//!
//! This tests a PATTERN, not AQELITH's runtime. See ADR-0003 and
//! docs/security/CAPABILITY_MODEL.md. Rust module privacy limits who can
//! construct a value; it does not stop other code in the same crate from
//! calling the root authority. That gap is a finding, recorded in the README.

/// Model of an in-memory token looked up by effect name (the pattern the Orison
/// audit describes). Any code that knows the string can construct it.
pub mod string_tokens {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct StringToken(pub String);

    impl StringToken {
        /// Public constructor: this is the forgeability.
        pub fn new(s: &str) -> Self {
            StringToken(s.to_string())
        }
    }

    #[derive(Default)]
    pub struct TokenTable {
        grants: HashMap<String, Vec<String>>,
    }

    impl TokenTable {
        pub fn grant(&mut self, token: &str, host: &str) {
            self.grants
                .entry(token.to_string())
                .or_default()
                .push(host.to_string());
        }

        pub fn check(&self, token: &StringToken, host: &str) -> bool {
            self.grants
                .get(&token.0)
                .is_some_and(|hosts| hosts.iter().any(|h| h == host))
        }
    }
}

/// Capability values: created only through an authority, attenuable, never widened.
pub mod capabilities {
    #[derive(Debug, PartialEq, Eq)]
    pub enum Denied {
        NotInScope { host: String },
    }

    /// Fields are private: values cannot be built with a struct literal outside this module.
    ///
    /// ```compile_fail
    /// use capability_values::capabilities::NetCap;
    /// let forged = NetCap { hosts: vec!["evil.example".to_string()] };
    /// ```
    #[derive(Debug, PartialEq, Eq)]
    pub struct NetCap {
        hosts: Vec<String>,
    }

    /// Host entry point. NOTE: any code in the process can call `Root::root()`.
    /// Rust cannot enforce "minted once by the host" here; AQELITH would need a
    /// language or runtime rule for it (open question, see README).
    pub struct Root {
        _private: (),
    }

    impl Root {
        pub fn root() -> Root {
            Root { _private: () }
        }

        pub fn net(&self, hosts: &[&str]) -> NetCap {
            NetCap {
                hosts: hosts.iter().map(|h| h.to_string()).collect(),
            }
        }
    }

    impl NetCap {
        pub fn connect(&self, host: &str) -> Result<String, Denied> {
            if self.hosts.iter().any(|h| h == host) {
                Ok(format!("connected:{host}"))
            } else {
                Err(Denied::NotInScope {
                    host: host.to_string(),
                })
            }
        }

        /// Attenuation: only a subset of the current hosts can be kept.
        pub fn narrow(&self, hosts: &[&str]) -> Result<NetCap, Denied> {
            for h in hosts {
                if !self.hosts.iter().any(|x| x == h) {
                    return Err(Denied::NotInScope {
                        host: h.to_string(),
                    });
                }
            }
            Ok(NetCap {
                hosts: hosts.iter().map(|h| h.to_string()).collect(),
            })
        }
    }

    /// A function that reaches the network must receive the capability as a parameter.
    pub fn fetch(net: &NetCap, host: &str) -> Result<String, Denied> {
        net.connect(host)
    }
}
