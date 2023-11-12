//! Parsing and recursively loading Python

use rustpython_parser::{parse, ast::{self, text_size::TextRange, TextSize}, Mode, ParseError};
use circ::circify::includer::Loader;

use log::debug;
use std::{path::{Path, PathBuf}, collections::HashMap};
use std::fs::File;
use std::io::Read;
use std::env::var_os;
use typed_arena::Arena;
use regex::Regex;

#[derive(Default)]
pub struct PyGadgets {
    path: PathBuf,
}

impl PyGadgets {
    pub fn new() -> Self {
        if let Some(p) = var_os("ZKPYC_STDLIB_PATH") {
            let p = PathBuf::from(p);
            if p.exists() {
                return Self { path: p };
            } else {
                panic!(
                    "PyGadgets: ZKPYC_STDLIB_PATH {:?} does not appear to exist",
                    p
                );
            }
        }

        let p = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert!(p.is_absolute());
        let stdlib_subdirs = vec![
            "zkpyc_stdlib/stdlib",
        ];
        for a in p.ancestors() {
            for subdir in &stdlib_subdirs {
                let mut q = a.to_path_buf();
                q.push(subdir);
                if q.exists() {
                    return Self { path: q };
                }
            }
        }
        panic!("Could not find ZKPyC stdlib from {}", p.display())
    }

    /// Turn `child`, relative to `parent` (or to the standard libary!), into an absolute path.
    pub fn canonicalize(&self, parent: &Path, child: &str) -> PathBuf {
        debug!("Looking for {} from {}", child, parent.display());
        let paths = [parent.to_path_buf(), self.path.clone()];
        for mut p in paths {
            p.push(child);
            debug!("Checking {}", p.display());
            if p.exists() || self.is_embed(&p) {
                return p;
            }
            if p.extension().is_some() {
                continue;
            }
            for ext in ["py"] {
                p.set_extension(ext);
                debug!("Checking {}", p.display());
                if p.exists() {
                    return p;
                }
            }
        }
        panic!("Could not find {} from {}", child, parent.display())
    }

    /// check if this path is the EMBED prototypes path
    pub fn is_embed<P: AsRef<Path>>(&self, p: P) -> bool {
        p.as_ref().starts_with(&self.path)
            && p.as_ref().file_stem().and_then(|s| s.to_str()) == Some("EMBED")
    }

}

/// Recursive Python module loader
pub struct PyLoad {
    sources: Arena<String>,
    stdlib: PyGadgets,
}

impl PyLoad {
    pub fn new() -> Self {
        Self {
            sources: Arena::new(),
            stdlib: PyGadgets::new(),
        }
    }

    /// Returns a map from file paths to parsed files.
    pub fn load<P: AsRef<Path>>(&self, p: &P) -> HashMap<PathBuf, ast::Mod> {
        self.recursive_load(p).unwrap()
    }

    pub fn stdlib(&self) -> &PyGadgets {
        &self.stdlib
    }
}

impl <'a> Loader for &'a PyLoad {
    type ParseError = ParseError;
    type AST = ast::Mod;

    fn parse<P: AsRef<Path>>(&self, p: &P) -> Result<Self::AST, Self::ParseError> {
        let mut s = String::new();
        File::open(p).unwrap().read_to_string(&mut s).unwrap();
        debug!("Parsing: {}", p.as_ref().display());
        filter_out_zk_ignore(&mut s);
        let s = self.sources.alloc(s);
        let ast = parse(&s, Mode::Module, p.as_ref().to_str().unwrap());
        if ast.is_err() {
            panic!("{}", ast.unwrap_err());
        }
        Ok(ast.unwrap())
    }

    fn includes<P: AsRef<Path>>(&self, ast: &Self::AST, p: &P) -> Vec<PathBuf> {
        let mut c = p.as_ref().to_path_buf();
        c.pop();        
        match ast {
            ast::Mod::Module(m) => {
                m.body
                    .iter()
                    .filter_map(|d| {
                        if let ast::Stmt::Import(stmt) = d {
                            // extract child path and canonicalize it
                            let ext = stmt.names[0].name
                                .replace(".", "/");
                            Some(self.stdlib.canonicalize(&c, ext.as_str()))
                        } else if let ast::Stmt::ImportFrom(stmt) = d {
                            let ext = stmt.module.clone()
                                .unwrap()
                                .replace(".", "/");
                            Some(self.stdlib.canonicalize(&c, ext.as_str()))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

pub fn filter_out_zk_ignore(s: &mut String) -> Vec<TextRange> {
    let re = Regex::new(r"(?i)#\s*zk_ignore\s*$").expect("Regex compilation failed");
    let lines: Vec<&str> = s.lines().collect(); // Split the string into lines

    let mut filtered_ranges: Vec<TextRange> = Vec::new();
    let mut current_line = TextSize::from(0);

    // Filter out lines that match the pattern and keep track of their ranges
    let filtered_lines: Vec<&str> = lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| {
            let is_filtered = !re.is_match(line);
            if !is_filtered {
                let line_length = TextSize::of(*line);
                filtered_ranges.push(TextRange::new(current_line, current_line + line_length));
            }
            // Add 1 to account for newline character
            current_line += TextSize::of(*line) + TextSize::from(1);
            is_filtered
        })
        .map(|(_, line)| line)
        .collect();

    // Join the filtered lines back into a new string
    let new_string = filtered_lines.join("\n");

    // Update the original string
    s.clear();
    s.push_str(&new_string);

    // Return the vector of TextRanges for filtered lines
    filtered_ranges
}
