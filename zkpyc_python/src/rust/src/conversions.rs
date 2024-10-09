use pyo3::{Python, types::{PyList, PyTuple}, PyResult, exceptions};
use zkpyc::utilities::r1cs::{Var, VarType};

// Function to convert Vec<Var> to Python object
pub fn vec_of_var_to_py(py: Python, input: Vec<Var>) -> &PyList {
    let py_list = PyList::new(
        py,
        input.into_iter().map(|var| {
            let ty = match var.ty() {
                VarType::Inst => 0,
                VarType::CWit => 1,
                VarType::RoundWit => 2,
                VarType::Chall => 3,
                VarType::FinalWit => 4,
            };
            let number = var.number();
            PyTuple::new(py, vec![ty, number])
        }),
    );
    py_list
}

// Function to convert PyList to Vec<Var>
pub fn py_to_vec_of_var(_py: Python, input: &PyList) -> PyResult<Vec<Var>> {
    let mut result = Vec::new();

    for item in input.iter() {
        let tuple = item.downcast::<PyTuple>()?;
        let ty_repr: usize = tuple.get_item(0)?.extract()?;
        let number: usize = tuple.get_item(1)?.extract()?;

        let ty = match ty_repr {
            0 => VarType::Inst,
            1 => VarType::CWit,
            2 => VarType::RoundWit,
            3 => VarType::Chall,
            4 => VarType::FinalWit,
            _ => return Err(exceptions::PyValueError::new_err("Invalid VarType")),
        };

        let var = Var::new(ty, number);
        result.push(var);
    }

    Ok(result)
}
