// Fun times ahead!
//
// Apparently, proc-macros don't play well with `cfg_attr` yet, and their
// combination is buggy. So we can't use cfg_attr to choose between
// `wasm-bindgen` and `structopt` depending on if we're building the CLI or the
// wasm API respectively. Instead, we have `build.rs` remove unwanted attributes
// for us by invoking `grep`.
//
// It's terrible! But it works for now.

/// Options for configuring `twiggy`.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[structopt(about = "\n`twiggy` is a code size profiler.\n\nIt analyzes a binary's call graph to answer questions like:\n\n* Why was this function included in the binary in the first place?\n\n* What is the retained size of this function? I.e. how much space\n  would be saved if I removed it and all the functions that become\n  dead code after its removal.\n\nUse `twiggy` to make your binaries slim!")]
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

    /// List the generic function monomorphizations that are contributing to
    /// code bloat.
    #[structopt(name = "monos")]
    Monos(Monos),

    /// Diff the old and new versions of a binary to see what sizes changed.
    #[structopt(name = "diff")]
    Diff(Diff),

    /// Find and display code and data that is not transitively referenced by
    /// any exports or public functions.
    #[structopt(name = "garbage")]
    Garbage(Garbage)
}

/// List the top code size offenders in a binary.
#[derive(Clone, Debug, Default)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Top {
    /// The path to the input binary to size profile.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// The maximum number of items to display.
    #[structopt(short = "n")]
    number: Option<u32>,

    /// Display retaining paths.
    #[structopt(short = "r", long = "retaining-paths")]
    retaining_paths: bool,

    /// Sort list by retained size, rather than shallow size.
    #[structopt(long = "retained")]
    retained: bool,
}

#[wasm_bindgen]
impl Top {
    /// Construct a new, default `Top`.
    pub fn new() -> Top {
        Top::default()
    }

    /// The maximum number of items to display.
    pub fn number(&self) -> u32 {
        self.number.unwrap_or(u32::MAX)
    }

    /// Display retaining paths.
    pub fn retaining_paths(&self) -> bool {
        self.retaining_paths
    }

    /// Sort list by retained size, rather than shallow size.
    pub fn retained(&self) -> bool {
        self.retained
    }

    /// Set the maximum number of items to display.
    pub fn set_number(&mut self, n: u32) {
        self.number = Some(n);
    }

    /// Set whether to display and compute retaining paths.
    pub fn set_retaining_paths(&mut self, do_it: bool) {
        self.retaining_paths = do_it;
    }

    /// Set whether to sort list by retained size, rather than shallow size.
    pub fn set_retained(&mut self, do_it: bool) {
        self.retained = do_it;
    }
}

/// Compute and display the dominator tree for a binary's call graph.
#[derive(Clone, Debug, Default)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Dominators {
    /// The path to the input binary to size profile.
    #[cfg(feature = "cli")]
   #[structopt(parse(from_os_str))]
    input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// The maximum depth to print the dominators tree.
    #[structopt(short = "d")]
    max_depth: Option<u32>,

    /// The maximum number of rows, regardless of depth in the tree, to display.
    #[structopt(short = "r")]
    max_rows: Option<u32>,

    /// The name of the function whose dominator subtree should be printed.
    #[structopt(long = "function", default_value = "")]
    func_name: String,
}

#[wasm_bindgen]
impl Dominators {
    /// Construct a new, default `Dominators`.
    pub fn new() -> Dominators {
        Dominators::default()
    }

    /// The maximum depth to print the dominators tree.
    pub fn max_depth(&self) -> u32 {
        self.max_depth.unwrap_or(u32::MAX)
    }

    /// The maximum number of rows, regardless of depth in the tree, to display.
    pub fn max_rows(&self) -> u32 {
        self.max_rows.unwrap_or(u32::MAX)
    }

    /// Set the maximum depth to print the dominators tree.
    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.max_depth = Some(max_depth);
    }

    /// Set the maximum number of rows, regardless of depth in the tree, to display.
    pub fn set_max_rows(&mut self, max_rows: u32) {
        self.max_rows = Some(max_rows);
    }

    /// The function whose subtree should be printed.
    pub fn func_name(&self) -> String {
        self.func_name.clone()
    }

    /// Set the function whose subtree should be printed.
    pub fn set_func_name(&mut self, func_name: &str) {
        self.func_name = func_name.to_string();
    }
}

/// Find and display the call paths to a function in the given binary's call
/// graph.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Paths {
    /// The path to the input binary to size profile.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// The functions to find call paths to.
    functions: Vec<String>,

    /// The maximum depth to print the paths.
    #[structopt(short = "d", default_value = "10")]
    max_depth: u32,

    /// The maximum number of paths, regardless of depth in the tree, to display.
    #[structopt(short = "r", default_value = "10")]
    max_paths: u32,
}

impl Default for Paths {
    fn default() -> Paths {
        Paths {
            #[cfg(feature = "cli")]
            input: Default::default(),
            #[cfg(feature = "cli")]
            output_destination: Default::default(),
            #[cfg(feature = "cli")]
            output_format: Default::default(),

            functions: Default::default(),
            max_depth: 10,
            max_paths: 10,
        }
    }
}

impl Paths {
    // TODO: wasm-bindgen doesn't support sending Vec<String> across the wasm
    // ABI boundary yet.

    /// The functions to find call paths to.
    pub fn functions(&self) -> &[String] {
        &self.functions
    }
}

#[wasm_bindgen]
impl Paths {
    /// Construct a new, default `Paths`.
    pub fn new() -> Paths {
        Paths::default()
    }

    /// Add a function to find call paths for.
    pub fn add_function(&mut self, function: String) {
        self.functions.push(function);
    }

    /// The maximum depth to print the paths.
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }

    /// The maximum number of paths, regardless of depth in the tree, to display.
    pub fn max_paths(&self) -> u32 {
        self.max_paths
    }

    /// Set the maximum depth to print the paths.
    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.max_depth = max_depth;
    }

    /// Set the maximum number of paths, regardless of depth in the tree, to display.
    pub fn set_max_paths(&mut self, max_paths: u32) {
        self.max_paths = max_paths;
    }
}

/// List the generic function monomorphizations that are contributing to
/// code bloat.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Monos {
    /// The path to the input binary to size profile.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// Hide individual monomorphizations and only show the generic functions.
    #[structopt(short = "g", long = "only-generics")]
    only_generics: bool,

    /// The maximum number of generics to list.
    #[structopt(short = "m", long = "max-generics", default_value = "10")]
    max_generics: u32,

    /// The maximum number of individual monomorphizations to list for each
    /// generic function.
    #[structopt(short = "n", long = "max-monos", default_value = "10")]
    max_monos: u32,
}

impl Default for Monos {
    fn default() -> Monos {
        Monos {
            #[cfg(feature = "cli")]
            input: Default::default(),
            #[cfg(feature = "cli")]
            output_destination: Default::default(),
            #[cfg(feature = "cli")]
            output_format: Default::default(),

            only_generics: false,
            max_generics: 10,
            max_monos: 10,
        }
    }
}

#[wasm_bindgen]
impl Monos {
    /// Construct a new, default `Monos`.
    pub fn new() -> Monos {
        Monos::default()
    }

    /// Hide individual monomorphizations and only show the generic functions.
    pub fn only_generics(&self) -> bool {
        self.only_generics
    }

    /// The maximum number of generics to list.
    pub fn max_generics(&self) -> u32 {
        self.max_generics
    }

    /// The maximum number of individual monomorphizations to list for each
    /// generic function.
    pub fn max_monos(&self) -> u32 {
        self.max_monos
    }

    /// Set whether to hide individual monomorphizations and only show the
    /// generic functions.
    pub fn set_only_generics(&mut self, do_it: bool) {
        self.only_generics = do_it;
    }

    /// Set the maximum number of generics to list.
    pub fn set_max_generics(&mut self, max: u32) {
        self.max_generics = max;
    }

    /// Set the maximum number of individual monomorphizations to list for each
    /// generic function.
    pub fn set_max_monos(&mut self, max: u32) {
        self.max_monos = max;
    }
}

/// Diff the old and new versions of a binary to see what sizes changed.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Diff {
    /// The path to the old version of the input binary.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    old_input: path::PathBuf,

    /// The path to the new version of the input binary.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    new_input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// The maximum number of items to display.
    #[structopt(short = "n", default_value = "20")]
    max_items: u32,
}

impl Default for Diff {
    fn default() -> Diff {
        Diff {
            #[cfg(feature = "cli")]
            old_input: Default::default(),
            #[cfg(feature = "cli")]
            new_input: Default::default(),
            #[cfg(feature = "cli")]
            output_destination: Default::default(),
            #[cfg(feature = "cli")]
            output_format: Default::default(),

            max_items: 20,
        }
    }
}

#[wasm_bindgen]
impl Diff {
    /// The maximum number of items to display.
    pub fn max_items(&self) -> u32 {
        self.max_items
    }

    /// Set the maximum number of items to display.
    pub fn set_max_items(&mut self, n: u32) {
        self.max_items = n;
    }
}

/// Find and display code and data that is not transitively referenced by any
/// exports or public functions.
#[derive(Clone, Debug)]
#[derive(StructOpt)]
#[wasm_bindgen]
pub struct Garbage {
    /// The path to the input binary to size profile.
    #[cfg(feature = "cli")]
    #[structopt(parse(from_os_str))]
    input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg(feature = "cli")]
    #[structopt(short = "o", default_value = "-")]
    output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg(feature = "cli")]
    #[structopt(short = "f", long = "format", default_value = "text")]
    output_format: traits::OutputFormat,

    /// The maximum number of items to display.
    #[structopt(short = "n", default_value = "10")]
    max_items: u32,
}

impl Default for Garbage {
    fn default() -> Garbage {
        Garbage {
            #[cfg(feature = "cli")]
            input: Default::default(),
            #[cfg(feature = "cli")]
            output_destination: Default::default(),
            #[cfg(feature = "cli")]
            output_format: Default::default(),

            max_items: 10,
        }
    }
}

#[wasm_bindgen]
impl Garbage {
    /// Construct a new, default `Garbage`
    pub fn new() -> Garbage {
        Garbage::default()
    }

    /// The maximum number of items to display.
    pub fn max_items(&self) -> u32 {
        self.max_items
    }

    /// Set the maximum number of items to display.
    pub fn set_max_items(&mut self, max: u32) {
        self.max_items = max;
    }
}
