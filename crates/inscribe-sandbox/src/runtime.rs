use std::io::{self, Write};

use inscribe_comptime::{ComptimeError, ComptimeResult, ComptimeValue, Runtime};

use crate::capability::Capability;
use crate::policy::SandboxPolicy;

#[derive(Debug, Clone)]
pub struct SandboxRuntime {
    policy: SandboxPolicy,
}

impl SandboxRuntime {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    fn require(&self, capability: Capability) -> ComptimeResult<()> {
        if self.policy.allows(capability) {
            Ok(())
        } else {
            Err(ComptimeError::new(format!(
                "sandbox policy denies {capability:?} capability"
            )))
        }
    }

    fn expect_string<'a>(&self, args: &'a [ComptimeValue], index: usize) -> ComptimeResult<&'a str> {
        match args.get(index) {
            Some(ComptimeValue::String(value)) => Ok(value),
            Some(other) => Err(ComptimeError::new(format!(
                "expected string argument, found {}",
                other.kind_name()
            ))),
            None => Err(ComptimeError::new("missing string argument")),
        }
    }

    fn expect_int(&self, args: &[ComptimeValue], index: usize) -> ComptimeResult<i64> {
        match args.get(index) {
            Some(ComptimeValue::Integer(value)) => Ok(*value),
            Some(other) => Err(ComptimeError::new(format!(
                "expected int argument, found {}",
                other.kind_name()
            ))),
            None => Err(ComptimeError::new("missing int argument")),
        }
    }
}

impl Runtime for SandboxRuntime {
    fn call(&self, name: &str, args: &[ComptimeValue]) -> ComptimeResult<ComptimeValue> {
        match name {
            "print_int" => {
                self.require(Capability::Stdout)?;
                let value = self.expect_int(args, 0)?;
                print!("{value}");
                Ok(ComptimeValue::Unit)
            }
            "print_bool" => {
                self.require(Capability::Stdout)?;
                let value = match args.get(0) {
                    Some(ComptimeValue::Bool(value)) => *value,
                    Some(other) => {
                        return Err(ComptimeError::new(format!(
                            "expected bool argument, found {}",
                            other.kind_name()
                        )))
                    }
                    None => return Err(ComptimeError::new("missing bool argument")),
                };
                if value {
                    print!("true");
                } else {
                    print!("false");
                }
                Ok(ComptimeValue::Unit)
            }
            "print_string" => {
                self.require(Capability::Stdout)?;
                let value = self.expect_string(args, 0)?;
                print!("{value}");
                Ok(ComptimeValue::Unit)
            }
            "print_newline" => {
                self.require(Capability::Stdout)?;
                println!();
                Ok(ComptimeValue::Unit)
            }
            "flush_stdout" => {
                self.require(Capability::Stdout)?;
                io::stdout()
                    .flush()
                    .map_err(|error| ComptimeError::new(format!("stdout flush failed: {error}")))?;
                Ok(ComptimeValue::Unit)
            }
            "read_int" => {
                self.require(Capability::Stdin)?;
                let mut buf = String::new();
                io::stdin()
                    .read_line(&mut buf)
                    .map_err(|error| ComptimeError::new(format!("stdin read failed: {error}")))?;
                let value = buf.trim().parse::<i64>().unwrap_or(0);
                Ok(ComptimeValue::Integer(value))
            }
            "string_length" => {
                let value = self.expect_string(args, 0)?;
                Ok(ComptimeValue::Integer(value.len() as i64))
            }
            "string_byte_at" => {
                let value = self.expect_string(args, 0)?;
                let index = self.expect_int(args, 1)?;
                if index < 0 {
                    return Ok(ComptimeValue::Integer(0));
                }
                let byte = value.as_bytes().get(index as usize).copied().unwrap_or(0);
                Ok(ComptimeValue::Integer(byte as i64))
            }
            "http_get" => {
                self.require(Capability::Network)?;
                let url = self.expect_string(args, 0)?;
                let response = ureq::get(url)
                    .call()
                    .map_err(|error| ComptimeError::new(format!("http_get failed: {error}")))?;
                let body = response
                    .into_string()
                    .map_err(|error| ComptimeError::new(format!("http_get failed: {error}")))?;
                Ok(ComptimeValue::String(body))
            }
            _ => Err(ComptimeError::new(format!(
                "unknown runtime declaration `{name}`"
            ))),
        }
    }
}
