//! Parsing and recursively loading Python

use rustpython_parser::{parse, ast::{self, text_size::TextRange, TextSize}, Mode, ParseError};
use circ::circify::includer::Loader;

use log::debug;
use std::{collections::{HashMap, VecDeque}, fs, path::{Path, PathBuf}};
use std::fs::File;
use std::io::Read;
use std::env::var_os;
use typed_arena::Arena;
use regex::Regex;
use dirs::data_dir;
use zkpyc_stdlib::StdLib;

use super::SourceInput;


#[derive(Default)]
pub struct PyGadgets {
    path: PathBuf,
}

impl PyGadgets {
    pub fn new() -> Self {
        // Get path from ZKPYC_STDLIB_PATH env var
        if let Some(p) = var_os("ZKPYC_STDLIB_PATH") {
            let p = PathBuf::from(p);
            if p.exists() {
                return Self { path: p };
            } else {
                panic!(
                    "ZKPYC_STDLIB_PATH {:?} does not appear to exist",
                    p
                );
            }
        }

        // If ZKPYC_STDLIB_PATH is not set then check data_dir
        let data_path = data_dir().unwrap();
        let stdlib_path = data_path.join("zkpyc");
        let version_file_path = stdlib_path.join("stdlib/version.txt");

        // Copy stdlib into data_path when run for the first time
        if !stdlib_path.exists() {
            debug!("First time run; copying stdlib into {}", &stdlib_path.display());
            StdLib::copy_stdlib(&stdlib_path.as_path());
            return Self { path: data_path };
        }

        // If stdlib exists in data_path, only modify if the version differs
        if let Ok(stored_version) = fs::read_to_string(&version_file_path) {
            if stored_version.trim() != StdLib::version() {
                debug!("Stdlib version has changed from {} to {}; updating stdlib...", stored_version.trim(), StdLib::version());
                StdLib::copy_stdlib(&stdlib_path.as_path());
                return Self { path: data_path };
            } else {
                debug!("Stdlib version has not changed; no need to update stdlib.");
                return Self { path: data_path };
            }
        } 

        // As fallback option, search through the current directory and its ancestors
        let p = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert!(p.is_absolute());
        let stdlib_subdirs = vec![
            "stdlib",
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
            if p.exists() {
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
    // pub fn is_embed<P: AsRef<Path>>(&self, p: P) -> bool {
    //     p.as_ref().starts_with(&self.path)
    //         && p.as_ref().file_stem().and_then(|s|s.to_str()) == Some("EMBED")
    // }

    pub fn is_embed<P: AsRef<Path>>(&self, p: P) -> bool {
        // For now we check it is either in the parent directory or in
        // the zkpyc directory (maybe make it customizable in the future).
        let p_ref = p.as_ref();
        p_ref.ends_with("EMBED.py") || p_ref.ends_with("zkpyc/stdlib/EMBED.py")
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
    pub fn load(&self, input: &SourceInput) -> HashMap<PathBuf, ast::Mod> {
        match input {
            SourceInput::Path(p) => self.recursive_load(p).unwrap(),
            SourceInput::String(s, p, n) => self.load_from_string(s, &p, n).unwrap(),
        }
    }

    /// Returns a map from file paths to parsed files, given a source string.
    fn load_from_string(
        &self,
        source: &str,
        working_dir: &Path,
        source_name: &str,
    ) -> Result<HashMap<PathBuf, ast::Mod>, <&PyLoad as Loader>::ParseError> {
        let mut ast_map = HashMap::default();
        let mut q = VecDeque::new();
        let fake_path = working_dir.join(source_name);

        let ast = self.parse_from_string(&source)?;
        for c in self.includes(&ast, &fake_path) {
            if !ast_map.contains_key(&c) {
                q.push_back(c);
            }
        }
        ast_map.insert(PathBuf::from(source_name), ast);

        while let Some(p) = q.pop_front() {
            if !ast_map.contains_key(&p) {
                // Join the recursively loaded results with the ast_map
                match self.recursive_load(&p) {
                    Ok(mut sub_map) => ast_map.extend(sub_map.drain()),
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(ast_map)
    }

    /// Parses source string directly.
    fn parse_from_string(&self, source: &str) -> Result<ast::Mod, <&PyLoad as Loader>::ParseError> {
        let mut s = String::from(source);
        filter_out_zk_ignore(&mut s);
        let s = self.sources.alloc(s);
        let ast = parse(&s, Mode::Module, "<embedded>");

        ast.map_err(|e| e.into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use ast::{Expr, Stmt};
    use fs::create_dir_all;
    use tempfile::TempDir;

    // Sample Python source code
    static MAIN_SRC: &str = r#"
from foo.bar import func

def foobar():
    return func("hello world")
    "#;

    static DEPENDENCY_1_SRC: &str = r#"
from dummy import dummy_str

def func(string):
    return dummy_str + string.upper()
    "#;

    static DEPENDENCY_2_SRC: &str = r#"dummy_str = "Person says: ""#;

    #[test]
    fn test_load_source_code_with_dependencies() -> Result<(), std::io::Error> {
        // Create temp paths
        let temp_dir = TempDir::new()?;
        let foo_bar_path = temp_dir.path().join("foo/bar.py");
        let dummy_str_path = temp_dir.path().join("foo/dummy.py");
    
        create_dir_all(foo_bar_path.parent().unwrap())?;
    
        // Write dependency files (foo/bar.py and foo/dummy.py)
        File::create(&foo_bar_path)?.write_all(DEPENDENCY_1_SRC.as_bytes())?;
        File::create(&dummy_str_path)?.write_all(DEPENDENCY_2_SRC.as_bytes())?;
    
        // Load using MAIN_SRC as a string (instead of from a file)
        let loader = PyLoad::new();
        let asts = loader.load(
            &SourceInput::String(MAIN_SRC.to_owned(), temp_dir.into_path(), "<embedded>".to_owned())
        );
    
        // 1. Check that the correct number of files were loaded
        assert_eq!(asts.len(), 3, "Expected three parsed ASTs (stdin + dependencies)");
    
        // 2. Check that the correct paths are stored in the hash map keys
        assert!(asts.keys().any(|p| p == Path::new("<embedded>")), "Missing source string");
        assert!(asts.keys().any(|p| p.ends_with("foo/bar.py")), "Missing foo/bar.py");
        assert!(asts.keys().any(|p| p.ends_with("foo/dummy.py")), "Missing foo/dummy.py");
    
        Ok(())
    }

    #[test]
    fn test_load_files_recursively() -> Result<(), std::io::Error> {
        // Create temp paths
        let temp_dir = TempDir::new()?;
        let main_path = temp_dir.path().join("main.py");
        let foo_bar_path = temp_dir.path().join("foo/bar.py");
        let dummy_str_path = temp_dir.path().join("foo/dummy.py");

        create_dir_all(foo_bar_path.parent().unwrap())?;

        // Write files
        File::create(&main_path)?.write_all(MAIN_SRC.as_bytes())?;
        File::create(&foo_bar_path)?.write_all(DEPENDENCY_1_SRC.as_bytes())?;
        File::create(&dummy_str_path)?.write_all(DEPENDENCY_2_SRC.as_bytes())?;

        // Parse the files recursively
        let loader = PyLoad::new();
        let asts = loader.load(&SourceInput::Path(main_path));

        // 1. Check that the three files were (recursively) loaded
        assert_eq!(asts.len(), 3, "Expected three parsed ASTs");

        // 2. Check that the correct paths are stored in the hash map keys
        assert!(asts.keys().any(|p| p.ends_with("main.py")), "Missing main.py");
        assert!(asts.keys().any(|p| p.ends_with("foo/bar.py")), "Missing foo/bar.py");
        assert!(asts.keys().any(|p| p.ends_with("foo/dummy.py")), "Missing foo/dummy.py");

        Ok(())
    }

    #[test]
    fn test_correctness_loaded_main_ast() -> Result<(), std::io::Error> {
        // Create temp paths
        let temp_dir = TempDir::new()?;
        let main_path = temp_dir.path().join("main.py");
        let foo_bar_path = temp_dir.path().join("foo/bar.py");
        let dummy_str_path = temp_dir.path().join("foo/dummy.py");

        create_dir_all(foo_bar_path.parent().unwrap())?;

        // Write files
        File::create(&main_path)?.write_all(MAIN_SRC.as_bytes())?;
        File::create(&foo_bar_path)?.write_all(DEPENDENCY_1_SRC.as_bytes())?;
        File::create(&dummy_str_path)?.write_all(DEPENDENCY_2_SRC.as_bytes())?;

        // Parse the files recursively
        let loader = PyLoad::new();
        let asts = loader.load(&SourceInput::Path(main_path));
        let main_ast = asts.iter().find(|(p, _)| p.ends_with("main.py"))
            .expect("AST for main.py not found")
            .1;

        // Check if AST is valid
        let mut has_import = false;
        let mut has_func = false;

        for stmt in &main_ast.as_module().unwrap().body {
            match stmt {
                Stmt::ImportFrom(import_stmt) if import_stmt.module.as_deref() == Some("foo.bar") => has_import = true,
                Stmt::FunctionDef(func) if func.name.as_str() == "foobar" => has_func = true,
                _ => {}
            }
        }

        assert!(has_import, "Module should contain an import statement");
        assert!(has_func, "Module should contain a function definition statement");

        Ok(())

    }

    #[test]
    fn test_correctness_loaded_foo_bar_ast() -> Result<(), std::io::Error> {
        // Create temp paths
        let temp_dir = TempDir::new()?;
        let foo_bar_path = temp_dir.path().join("foo/bar.py");
        let dummy_str_path = temp_dir.path().join("foo/dummy.py");

        create_dir_all(foo_bar_path.parent().unwrap())?;

        // Write files
        File::create(&foo_bar_path)?.write_all(DEPENDENCY_1_SRC.as_bytes())?;
        File::create(&dummy_str_path)?.write_all(DEPENDENCY_2_SRC.as_bytes())?;

        // Parse the files recursively
        let loader = PyLoad::new();
        let asts = loader.load(&SourceInput::Path(foo_bar_path));
        let foo_bar_ast = asts.iter().find(|(p, _)| p.ends_with("foo/bar.py"))
            .expect("AST for foo/bar.py not found")
            .1;

        // Check if AST is valid
        let mut has_import = false;
        let mut has_func = false;
        let mut has_binop = false;

        for stmt in &foo_bar_ast.as_module().unwrap().body {
            match stmt {
                Stmt::ImportFrom(import_stmt) if import_stmt.module.as_deref() == Some("dummy") => has_import = true,
                Stmt::FunctionDef(func) if func.name.as_str() == "func" => {
                    has_func = true;
                    if let Some(Stmt::Return(ret_stmt)) = func.body.first() {
                        if let Some(Expr::BinOp(_)) = ret_stmt.value.as_deref() {
                            has_binop = true;
                        }
                    }
                }
                _ => {}
            }
        }

        assert!(has_import, "Module should contain an import statement");
        assert!(has_func, "Module should contain a function definition statement");
        assert!(has_binop, "Module should contain a binary operation expression");

        Ok(())
    }

    #[test]
    fn test_correctness_loaded_dummy_str_ast() -> Result<(), std::io::Error> {
        // Create temp paths
        let temp_dir = TempDir::new()?;
        let dummy_str_path = temp_dir.path().join("foo/dummy.py");

        create_dir_all(dummy_str_path.parent().unwrap())?;

        // Write files
        File::create(&dummy_str_path)?.write_all(DEPENDENCY_2_SRC.as_bytes())?;

        // Parse the files recursively
        let loader = PyLoad::new();
        let asts = loader.load(&SourceInput::Path(dummy_str_path));
        let dummy_str_ast = asts.iter().find(|(p, _)|p.ends_with("foo/dummy.py"))
            .expect("AST for foo/dummy.py not found")
            .1;

        // Check if AST is valid
        let mut has_str = false;
        match &dummy_str_ast.as_module().unwrap().body[0] {
            Stmt::Assign(assign_stmt) if &assign_stmt.targets[0].as_name_expr().unwrap().id == "dummy_str" => has_str = true,
            _ => (),
        }

        assert!(has_str, "Module should contain a string expression");
        Ok(())
    }

}