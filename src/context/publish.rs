use error::Error;
use publish::npm_publish;

use super::Context;

// This file contains the implementation of the `publish` subcommand.

impl Context {
    pub fn publish(&mut self) -> Result<(), Error> {
        let publish_res = npm_publish(&self.path);
        if publish_res.is_ok() {
            self.pbar.message("💥  published your package!");
        }
        publish_res
    }
}
