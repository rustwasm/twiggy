/// Demangle names.
pub(super) fn demangle(s: &str) -> Option<String> {
    if let Ok(sym) = rustc_demangle::try_demangle(s) {
        return Some(sym.to_string());
    }

    // If the Rust demangle failed, we'll try C or C++.  C++
    // symbols almost all start with the prefixes "_Z", "__Z", and
    // ""_GLOBAL_", except for a special case.
    //
    // Per cpp_mangle::ast::MangledName::parse:
    //
    // > The libiberty tests also specify that a type can be top level,
    // > and they are not prefixed with "_Z".
    //
    // Therefore cpp_demangle will parse unmangled symbols, at
    // least sometimes incorrectly (e.g. with OpenSSL's RC4
    // function, which is incorrectly parsed as a type ctor/dtor),
    // which confuses a subsequent `demangle` function, resulting
    // in panic.
    //
    // To avoid that, only pass C++-mangled symbols to the C++
    // demangler
    if !s.starts_with("_Z") && !s.starts_with("__Z") && !s.starts_with("_GLOBAL_") {
        return Some(s.to_string());
    }

    if let Ok(sym) = cpp_demangle::Symbol::new(s) {
        return Some(sym.to_string());
    }

    None
}
