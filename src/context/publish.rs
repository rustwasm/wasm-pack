use error::Error;
use command::npm_publish;

use super::Context;

// This file contains the implementation of the `publish` subcommand.

impl Context {
    pub fn publish(&mut self) -> Result<(), Error> {
        npm_publish(&self.path)
    }
}
