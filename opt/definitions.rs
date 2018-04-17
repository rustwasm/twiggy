// Fun times ahead!
//
// Apparently, proc-macros don't play well with `cfg_attr` yet, and their
// combination is buggy. So we can't use cfg_attr to choose between
// `wasm-bindgen` and `structopt` depending on if we're building the CLI or the
// wasm API respectively. Instead, we have `build.rs` remove unwanted attributes
// for us by invoking `grep`.
//
// It's terrible! But it works for now.

/// Options for configuring `svelte`.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[structopt(about = "\n`svelte` is a code size profiler.\n\nIt analyzes a binary's call graph to answer questions like:\n\n* Why was this function included in the binary in the first place?\n\n* What is the retained size of this function? I.e. how much space\n  would be saved if I removed it and all the functions that become\n  dead code after its removal.\n\nUse `svelte` to make your binaries slim!")]
pub enum Options {
    /// List the top code size offenders in a binary.
    #[structopt(name = "top")]
    Top(Top),

    /// Compute and display the dominator tree for a binary's call graph.
    #[structopt(name = "dominators")]
    Dominators(Dominators),

    /// Find and display the call paths to a function in the given binary's call
    /// graph.
    #[structopt(name = "paths")]
    Paths(Paths),
}

/// List the top code size offenders in a binary.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Top {
    /// The path to the input binary to size profile.
    #[structopt(parse(from_os_str))]
    pub input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[structopt(short = "o", default_value = "-")]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[structopt(short = "f", long = "format", default_value = "text")]
    pub output_format: traits::OutputFormat,

    /// The maximum number of items to display.
    #[structopt(short = "n")]
    pub number: Option<u32>,

    /// Display retaining paths.
    #[structopt(short = "r", long = "retaining-paths")]
    pub retaining_paths: bool,

    /// Sort list by retained
    #[structopt(long = "retained")]
    pub retained: bool,
}

/// Compute and display the dominator tree for a binary's call graph.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Dominators {
    /// The path to the input binary to size profile.
    #[structopt(parse(from_os_str))]
    pub input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[structopt(short = "o", default_value = "-")]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[structopt(short = "f", long = "format", default_value = "text")]
    pub output_format: traits::OutputFormat,

    /// The maximum depth to print the dominators tree.
    #[structopt(short = "d")]
    pub max_depth: Option<usize>,

    /// The maximum number of rows, regardless of depth in the tree, to display.
    #[structopt(short = "r")]
    pub max_rows: Option<usize>,
}

/// Find and display the call paths to a function in the given binary's call
/// graph.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Paths {
    /// The path to the input binary to size profile.
    #[structopt(parse(from_os_str))]
    pub input: path::PathBuf,

    /// The functions to find call paths to.
    pub functions: Vec<String>,

    /// The destination to write the output to. Defaults to `stdout`.
    #[structopt(short = "o", default_value = "-")]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[structopt(short = "f", long = "format", default_value = "text")]
    pub output_format: traits::OutputFormat,

    /// The maximum depth to print the paths.
    #[structopt(short = "d", default_value = "10")]
    pub max_depth: usize,

    /// The maximum number of paths, regardless of depth in the tree, to display.
    #[structopt(short = "r", default_value = "10")]
    pub max_paths: usize,
}
