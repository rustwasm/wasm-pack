use command::npm_pack;
use error::Error;

use super::Context;

// This file contains the implementation of the `pack` subcommand.

impl Context {
    pub fn pack(&mut self) -> Result<(), Error> {
        let pack_res = npm_pack(&self.path);
        if pack_res.is_ok() {
            self.pbar.message("🎒  packed up your package!");
        }
        pack_res
    }
}
