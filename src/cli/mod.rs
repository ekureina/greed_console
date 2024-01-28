use std::ffi::OsString;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    pub campaigns: Vec<OsString>,
}
