// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use clap::{ArgAction, Parser};

use crate::options;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayoutAlgorithm {
    None,
    Dot,
    Neato,
    Twopi,
    Circo,
    Fdp,
    Sfdp,
}

impl FromStr for LayoutAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "dot" => Ok(Self::Dot),
            "neato" => Ok(Self::Neato),
            "twopi" => Ok(Self::Twopi),
            "circo" => Ok(Self::Circo),
            "fdp" => Ok(Self::Fdp),
            "sfdp" => Ok(Self::Sfdp),
            _ => Err("Unrecognized layout"),
        }
    }
}

impl ToString for LayoutAlgorithm {
    fn to_string(&self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Dot => "dot".to_owned(),
            Self::Neato => "neato".to_owned(),
            Self::Twopi => "twopi".to_owned(),
            Self::Circo => "circo".to_owned(),
            Self::Fdp => "fdp".to_owned(),
            Self::Sfdp => "sfdp".to_owned(),
        }
    }
}

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "GenerateSelectionOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: options::general::Options,

    #[command(flatten)]
    pub project: options::project::Options,

    #[command(flatten)]
    pub selection: options::selection::Options,

    /// Require graph to be acyclic
    #[arg(long = "acyclic", action = ArgAction::SetTrue, conflicts_with = "focus_on")]
    pub acyclic: bool,

    /// The graph layout algorithm to use
    /// (e.g. none, dot, neato, twopi, circo, fdp, sfdp).
    #[arg(long = "layout", default_value = "neato")]
    pub layout: LayoutAlgorithm,

    // The `modules` and `no_modules` args might look like they have their
    // documentation comments and clap-args mixed up, but they have to be
    // that way in order to work-around a limitation of clap:
    // https://jwodder.github.io/kbits/posts/clap-bool-negate/
    // https://github.com/clap-rs/clap/issues/815
    /// Exclude modules (e.g. `mod foo`, `mod foo {}`).
    #[clap(long = "no-modules", action = ArgAction::SetFalse)]
    pub modules: bool,

    // The `modules` and `no_modules` args might look like they have their
    // documentation comments and clap-args mixed up, but they have to be
    // that way in order to work-around a limitation of clap:
    // https://jwodder.github.io/kbits/posts/clap-bool-negate/
    // https://github.com/clap-rs/clap/issues/815
    /// Include modules (e.g. `mod foo`, `mod foo {}`). [default]
    #[clap(long = "modules", action = ArgAction::SetTrue, overrides_with = "modules")]
    pub no_modules: (),

    /// Include used modules and types
    #[arg(long = "uses")]
    pub uses: bool,

    /// Exclude used modules and types [default]
    #[arg(long = "no-uses", action = ArgAction::SetFalse, overrides_with = "uses")]
    pub no_uses: (),

    /// Include used modules and types from extern crates
    #[arg(long = "externs")]
    pub externs: bool,

    /// Exclude used modules and types from extern crates [default]
    #[arg(long = "no-externs", action = ArgAction::SetFalse, overrides_with = "externs")]
    pub no_externs: (),
}