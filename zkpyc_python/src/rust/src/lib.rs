mod utilities;
mod compiler;
mod backend;
mod ff_constants;
mod conversions;

use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _rust(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_submodule(compiler::create_submodule(py)?)?;
    m.add_submodule(backend::create_submodule(py)?)?;
    Ok(())
}
