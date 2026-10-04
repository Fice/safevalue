#![deny(deprecated)]
#![allow(unused)]
extern crate safevalue;
use safevalue::{unsafe_marker, unsafe_marker_no_send};

// the new syntax is fine
unsafe_marker!(pub struct NewSyntax);

// the old one is deprecated
unsafe_marker!(pub OldSyntax);
unsafe_marker_no_send!(pub OldNoSend);

pub fn main() {}
