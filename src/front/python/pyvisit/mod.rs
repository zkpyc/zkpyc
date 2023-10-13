//! AST Walker based on rustpython_ast visitor implementation

mod walkfns;
mod pyvmut;

pub use rustpython_parser::ast::Visitor;

pub struct PyVisitorError(pub String);
pub type PyResult<PyTerm> = Result<PyTerm, PyVisitorError>;
pub type PyVisitorResult = PyResult<()>;

pub use pyvmut::PyVisitorMut;

impl From<String> for PyVisitorError {
    fn from(f: String) -> Self {
        Self(f)
    }
}