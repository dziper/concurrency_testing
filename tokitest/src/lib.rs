#![allow(dead_code)]

// Hidden module for macro support
#[doc(hidden)]
pub mod macros {
    pub use futures::future::Either;
}

pub mod controller;
pub mod label_spec;
pub mod prelude;

pub use crate::label_spec::{
    OrLabel,
    RegexLabel,
    StringLabel,
    RepeatedLabel,
};

pub use tokitest_macro::{
    start_tokitest,
    run_to,
    testable,
    testable_struct,
    label,
    spawn,
    spawn_join_set,
    call,
    network_call,
    isolate
};