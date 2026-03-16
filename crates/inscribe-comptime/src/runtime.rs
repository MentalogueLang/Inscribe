use crate::boundary::{ComptimeResult, ComptimeValue};

pub trait Runtime: Send + Sync {
    fn call(&self, name: &str, args: &[ComptimeValue]) -> ComptimeResult<ComptimeValue>;
}
