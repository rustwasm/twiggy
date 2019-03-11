//! Options for running `twiggy`.

#![deny(missing_debug_implementations)]

use cfg_if::cfg_if;
use twiggy_traits as traits;

cfg_if! {
    if #[cfg(feature = "cli")] {
        include!(concat!(env!("OUT_DIR"), "/cli.rs"));
    } else if #[cfg(feature = "wasm")] {
        use wasm_bindgen::prelude::*;
        include!(concat!(env!("OUT_DIR"), "/wasm.rs"));
    } else {
        compile_error!("Must enable one of either `cli` or `wasm` features");
    }
}

use std::u32;

cfg_if! {
    if #[cfg(feature = "cli")] {
        use std::fs;
        use std::io;
        use std::path;
        use std::str::FromStr;

        /// Options that are common to all commands.
        pub trait CommonCliOptions {
            /// Get the input file path.
            fn input(&self) -> &path::Path;

            /// Get the input data parse mode.
            fn parse_mode(&self) -> traits::ParseMode;

            /// Get the output destination.
            fn output_destination(&self) -> &OutputDestination;

            /// Get the output format.
            fn output_format(&self) -> traits::OutputFormat;
        }

        impl CommonCliOptions for Options {
            fn input(&self) -> &path::Path {
                match *self {
                    Options::Top(ref top) => top.input(),
                    Options::Dominators(ref doms) => doms.input(),
                    Options::Paths(ref paths) => paths.input(),
                    Options::Monos(ref monos) => monos.input(),
                    Options::Diff(ref diff) => diff.input(),
                    Options::Garbage(ref garbo) => garbo.input(),
                }
            }

            fn parse_mode(&self) -> traits::ParseMode {
                match *self {
                    Options::Top(ref top) => top.parse_mode(),
                    Options::Dominators(ref doms) => doms.parse_mode(),
                    Options::Paths(ref paths) => paths.parse_mode(),
                    Options::Monos(ref monos) => monos.parse_mode(),
                    Options::Diff(ref diff) => diff.parse_mode(),
                    Options::Garbage(ref garbo) => garbo.parse_mode(),
                }
            }

            fn output_destination(&self) -> &OutputDestination {
                match *self {
                    Options::Top(ref top) => top.output_destination(),
                    Options::Dominators(ref doms) => doms.output_destination(),
                    Options::Paths(ref paths) => paths.output_destination(),
                    Options::Monos(ref monos) => monos.output_destination(),
                    Options::Diff(ref diff) => diff.output_destination(),
                    Options::Garbage(ref garbo) => garbo.output_destination(),
                }
            }

            fn output_format(&self) -> traits::OutputFormat {
                match *self {
                    Options::Top(ref top) => top.output_format(),
                    Options::Dominators(ref doms) => doms.output_format(),
                    Options::Paths(ref paths) => paths.output_format(),
                    Options::Monos(ref monos) => monos.output_format(),
                    Options::Diff(ref diff) => diff.output_format(),
                    Options::Garbage(ref garbo) => garbo.output_format(),
                }
            }
        }

        impl CommonCliOptions for Top {
            fn input(&self) -> &path::Path {
                &self.input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        impl CommonCliOptions for Dominators {
            fn input(&self) -> &path::Path {
                &self.input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        impl CommonCliOptions for Paths {
            fn input(&self) -> &path::Path {
                &self.input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        impl CommonCliOptions for Monos {
            fn input(&self) -> &path::Path {
                &self.input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        impl CommonCliOptions for Diff {
            fn input(&self) -> &path::Path {
                &self.old_input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        impl Diff {
            /// The path to the new version of the input binary.
            pub fn new_input(&self) -> &path::Path {
                &self.new_input
            }
        }

        impl CommonCliOptions for Garbage {
            fn input(&self) -> &path::Path {
                &self.input
            }

            fn parse_mode(&self) -> traits::ParseMode {
                self.parse_mode
            }

            fn output_destination(&self) -> &OutputDestination {
                &self.output_destination
            }

            fn output_format(&self) -> traits::OutputFormat {
                self.output_format
            }
        }

        /// Where to output results.
        #[derive(Clone, Debug)]
        pub enum OutputDestination {
            /// Emit the results to `stdout`.
            Stdout,

            /// Write the results to a file at the given path.
            Path(path::PathBuf),
        }

        impl Default for OutputDestination {
            fn default() -> OutputDestination {
                OutputDestination::Stdout
            }
        }

        impl FromStr for OutputDestination {
            type Err = traits::Error;

            fn from_str(s: &str) -> Result<Self, traits::Error> {
                if s == "-" {
                    Ok(OutputDestination::Stdout)
                } else {
                    let path = path::PathBuf::from(s.to_string());
                    Ok(OutputDestination::Path(path))
                }
            }
        }

        impl OutputDestination {
            /// Open the output destination as an `io::Write`.
            pub fn open(&self) -> Result<Box<io::Write>, traits::Error> {
                Ok(match *self {
                    OutputDestination::Path(ref path) => {
                        Box::new(io::BufWriter::new(fs::File::create(path)?)) as Box<io::Write>
                    }
                    OutputDestination::Stdout => Box::new(io::stdout()) as Box<io::Write>,
                })
            }
        }
    }
}
