//! The Python (3.12 subset) front-end

mod parser;
mod term;
pub mod pyvisit;

use super::{FrontEnd, Mode, proof::PROVER_ID};
use circ::circify::{CircError, Circify, Loc, Val};
use circ::ir::proof::ConstraintMetadata;
use circ::cfg::cfg;
use circ::ir::term::*;
use circ::term;
use rug::Integer;
use rustpython_parser::ast::bigint::BigInt;
use rustpython_parser::ast::{Ranged, text_size::TextRange, TextSize};
use log::{debug, trace};
use parser::filter_out_zk_ignore;

use std::cell::{Cell, RefCell};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use rustpython_parser::ast as ast;
use rustpython_parser;

use term::*;

// garbage collection increment for adaptive GC threshold
const GC_INC: usize = 32;

/// Inputs to the Python compiler
pub struct Inputs {
    /// The file to look for the entry point. e.g. the `main` function.
    pub file: PathBuf,
    // The entry point.
    pub entry_point: String,
    /// Mode to generate for (MPC or proof).
    pub mode: Mode,
}

pub struct PythonFE;

impl FrontEnd for PythonFE {
    type Inputs = Inputs;
    fn gen(i: Inputs) -> Computations {
        debug!(
            "Starting Python front-end, field: {}",
            Sort::Field(cfg().field().clone())
        );
        let loader = parser::PyLoad::new();
        let asts = loader.load(&i.file);
        // need to figure out how to create python config
        let mut g = PyGen::new(asts, i.mode, loader.stdlib(), cfg().zsharp.isolate_asserts);
        g.visit_files(&i.entry_point);
        g.file_stack_push(i.file);
        // no generics for now
        g.entry_fn(&i.entry_point);
        g.file_stack_pop();

        let mut cs = Computations::new();
        let main_comp = std::rc::Rc::try_unwrap(g.into_circify().consume())
            .unwrap_or_else(|rc| (*rc).clone())
            .into_inner();
        cs.comps.insert(i.entry_point.clone(), main_comp);
        cs
    }
}

impl PythonFE {
    pub fn interpret(i: Inputs) -> PyTerm {
        let loader = parser::PyLoad::new();
        let asts = loader.load(&i.file);
        // like before, figure out cfg() zsharp part
        let mut g = PyGen::new(asts, i.mode, loader.stdlib(), cfg().zsharp.isolate_asserts);
        g.visit_files(&i.entry_point);
        g.file_stack_push(i.file);
        g.const_entry_fn(&i.entry_point)
    }
}

struct PyGen<'a> {
    circ: RefCell<Circify<Python>>,
    stdlib: &'a parser::PyGadgets,
    asts: HashMap<PathBuf, ast::Mod>,
    file_stack: RefCell<Vec<PathBuf>>,
    functions: HashMap<PathBuf, HashMap<String, ast::StmtFunctionDef>>,
    classes_and_tys: HashMap<
        PathBuf,
        HashMap<String, Result<ast::StmtClassDef, ast::StmtTypeAlias>>,
    >,
    // not yet sure if we need this
    constants: HashMap<PathBuf, HashMap<String, (ast::Expr, PyTerm)>>,
    import_map: HashMap<PathBuf, HashMap<String, (PathBuf, String)>>,
    mode: Mode,
    cvars_stack: RefCell<Vec<Vec<HashMap<String, PyTerm>>>>,
    crets_stack: RefCell<Vec<PyTerm>>,
    curr_func: RefCell<String>,
    lhs_ty: RefCell<Option<Ty>>,
    ret_ty_stack: RefCell<Vec<Ty>>,
    gc_depth_estimate: Cell<usize>,
    assertions: RefCell<Vec<Term>>,
    isolate_asserts: bool,
}

impl<'a> Drop for PyGen<'a> {
    fn drop(&mut self) {
        use std::mem::take;

        // drop all fields that contain PyTerm or Ty
        drop(take(&mut self.constants));
        drop(self.cvars_stack.take());
        drop(self.crets_stack.take());
        drop(self.lhs_ty.take());
        drop(self.ret_ty_stack.take());

        // force garbage collection
        garbage_collect();
    }
}

enum PyTarget {
    Member(String),
    Idx(PyTerm),
}

enum Literal {
    PyLiteral(ast::ExprConstant),
    Field(String, TextRange),
}

fn loc_store(class_: PyTerm, loc: &[PyTarget], val: PyTerm) -> Result<PyTerm, String> {
    match loc.first() {
        None => Ok(val),
        Some(PyTarget::Member(field)) => {
            let old_inner = field_select(&class_, field)?;
            let new_inner = loc_store(old_inner, &loc[1..], val)?;
            field_store(class_, field, new_inner)
        }
        Some(PyTarget::Idx(idx)) => {
            let old_inner = array_select(class_.clone(), idx.clone())?;
            let new_inner = loc_store(old_inner, &loc[1..], val)?;
            array_store(class_, idx.clone(), new_inner)
        }
    }
}

enum PyVis {
    Public,
    Private(u8),
    // not a feature yet
    Committed,
}

impl<'a> PyGen<'a> {
    fn new(
        asts: HashMap<PathBuf, ast::Mod>,
        mode: Mode,
        stdlib: &'a parser::PyGadgets,
        isolate_asserts: bool,
    ) -> Self {
        let this = Self {
            circ: RefCell::new(Circify::new(Python::new())),
            asts,
            stdlib,
            file_stack: Default::default(),
            functions: HashMap::new(),
            classes_and_tys: HashMap::new(),
            constants: HashMap::new(),
            import_map: HashMap::new(),
            mode,
            cvars_stack: Default::default(),
            crets_stack: Default::default(),
            curr_func: RefCell::new(String::from("<module>")),
            lhs_ty: Default::default(),
            ret_ty_stack: Default::default(),
            gc_depth_estimate: Cell::new(2 * GC_INC),
            assertions: Default::default(),
            isolate_asserts,
        };
        this.circ
            .borrow()
            .cir_ctx()
            .cs
            .borrow_mut()
            .metadata
            .add_prover_and_verifier();
        this
    }

    fn into_circify(self) -> Circify<Python> {
        self.circ.replace(Circify::new(Python::new()))
    }

    fn err<E: Display>(&self, e: E, s: &TextRange) -> ! {
        let range = range_before_filter(s, self.cur_path());
        let line = line_from_range(range, self.cur_path());
        
        // println!("ZKPyC Compilation Error -- Traceback:");
        // println!("\tFile {}, line {:?}, in {}", self.cur_path().canonicalize().unwrap().display(), line, self.curr_func.borrow());
        // println!("{}", range_to_string(s, self.cur_path()));
        // println!("Error: {e}");
        // std::process::exit(1)
        panic!(
            "ZKPyC Compilation Error -- Traceback:\n\
            \tFile {}, line {:?}, in {}\n\
            {}\n\
            Error: {}",
            self.cur_path().canonicalize().unwrap().display(),
            line,
            self.curr_func.borrow(),
            range_to_string(s, self.cur_path()),
            e
        );
    }

    fn unwrap<PyTerm, E: Display>(&self, r: Result<PyTerm, E>, s: &TextRange) -> PyTerm {
        r.unwrap_or_else(|e| self.err(e, s))
    }

    fn builtin_call(f_name: &str, mut args: Vec<PyTerm>) -> Result<PyTerm, String> {
        debug!("Builtin Call: {}", f_name);
        match f_name {
            "int_to_bits" => {
                if args.len() != 1 {
                    Err(format!(
                        "Got {} args to EMBED/{}, expected 1",
                        args.len(),
                        f_name
                    ))
                } else {
                    uint_to_bits(args.pop().unwrap())
                }
            }
            "int_from_bits" => {
                if args.len() != 1 {
                    Err(format!(
                        "Got {} args to EMBED/{}, expected 1",
                        args.len(),
                        f_name
                    ))
                } else {
                    uint_from_bits(args.pop().unwrap())
                }
            }
            "int_to_field" => {
                if args.len() != 1 {
                    Err(format!(
                        "Got {} args to EMBED/{}, expected 1",
                        args.len(),
                        f_name
                    ))
                } else {
                    uint_to_field(args.pop().unwrap())
                }
            }
            "unpack" => {
                if args.len() != 2 {
                    Err(format!(
                        "Got {} args to EMBED/unpack, expected 2",
                        args.len()
                    ))
                } else {
                    let arg: PyTerm = args.pop().unwrap();
                    let nbits =
                        const_int(arg)?
                            .to_usize()
                            .ok_or_else(|| {
                                "builtin_call failed to convert unpack's N to usize".to_string()
                            })?;
                    field_to_bits(args.pop().unwrap(), nbits)
                }
            }
            "pack" => {
                if args.len() != 1 {
                    Err(format!(
                        "Got {} args to EMBED/unpack, expected 1",
                        args.len()
                    ))
                } else {
                    field_from_bits(args.pop().unwrap())
                }
            }
            "bit_array_le" => {
                if args.len() != 2 {
                    Err(format!(
                        "Got {} args to EMBED/bit_array_le, expected 2",
                        args.len()
                    ))
                } else {
                    let arg: PyTerm = args.pop().unwrap();
                    let nbits =
                        const_int(arg)?
                            .to_usize()
                            .ok_or_else(|| {
                                "builtin_call failed to convert bit_array_le's N to usize"
                                    .to_string()
                            })?;

                    let second_arg = args.pop().unwrap();
                    let first_arg = args.pop().unwrap();
                    bit_array_le(first_arg, second_arg, nbits)
                }
            }
            "get_field_size" => {
                if !args.is_empty() {
                    Err(format!(
                        "Got {} args to EMBED/get_field_size, expected 0",
                        args.len()
                    ))
                } else {
                    Ok(uint_lit(cfg().field().modulus().significant_bits(), 32))
                }
            }
            "sum" => {
                if args.len() != 1 {
                    Err(format!(
                        "Got {} args to EMBED/get_field_size, expected 1",
                        args.len()
                    ))
                } else {
                    let arg: PyTerm = args.pop().unwrap();
                    match arg.ty {
                        Ty::Array(_, _) => {
                            let terms: Vec<PyTerm> = arg.unwrap_array()?;
                            let (init_result, rest) = terms.split_first().unwrap();
                            let final_result = rest.iter().try_fold(init_result.clone(), |prev_result, term| {
                                add(prev_result.clone(), term.clone())
                            });
                            final_result
                        }
                        t => Err(format!("Expected array, got {} instead", t)),
                    }
                }
            }
            _ => Err(format!("Unknown or unimplemented builtin '{f_name}'")),
        }
    }

    fn assign_impl_<const IS_CNST: bool>(
        &self,
        name: &str,
        target: Option<ast::Expr>,
        val: PyTerm,
        strict: bool,
    ) -> Result<(), String> {
        let pytargets: Vec<PyTarget>;
        if let Some(t) = target {
            pytargets = self.pytargets_impl_::<IS_CNST>(&t).unwrap();
        } else {
            pytargets = Vec::new();
        }
        let old = if IS_CNST {
            self.cvar_lookup(name)
                .ok_or_else(|| format!("Assignment failed: no const variable {name}"))?
        } else {
            self.circ_get_value(Loc::local(name.to_string()))
                .map_err(|e| format!("{e}"))?
                .unwrap_term()
        };
        let new =
            loc_store(old, &pytargets[..], val)
                .and_then(|n| if strict { const_val(n) } else { Ok(n) })?;
        debug!("Assign: {}", name);
        if IS_CNST {
            self.cvar_assign(name, new)
        } else {
            self.circ_assign(Loc::local(name.to_string()), Val::Term(new))
                .map_err(|e| format!("{e}"))
                .map(|_| ())
        }
    }

    fn pytargets_impl_<const IS_CNST: bool>(
        &self,
        target: &ast::Expr,
    ) -> Result<Vec<PyTarget>, String> {
        match target {
            ast::Expr::Subscript(s) => {
                let mut target: Vec<PyTarget> = self.pytargets_impl_::<IS_CNST>(&s.value)?;
                target.push(
                    self.expr_impl_::<IS_CNST>(&s.slice).map(PyTarget::Idx)?
                );
                Ok(target)
            }
            ast::Expr::Attribute(a) => {
                let mut target: Vec<PyTarget> = self.pytargets_impl_::<IS_CNST>(&a.value)?;
                target.push(
                    PyTarget::Member(a.attr.to_string())
                );
                Ok(target)
            }
            ast::Expr::Name(_) => {
                let vec: Vec<PyTarget> = Vec::new();
                Ok(vec)
            }
            err => {
                self.err(
                    format!("Only arrays and object members can be accessed."),
                    &err.range(),
                )
            }
        }
    }

    fn literal_(&self, e: &Literal) -> Result<PyTerm, String> {
        match &e {
            Literal::PyLiteral(c) => match &c.value {
                ast::Constant::None => {
                    self.err(
                        format!(
                            "There is no support yet for None literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Bool(v) => Ok(py_bool_lit(*v)),
                ast::Constant::Str(_) => {
                    self.err(
                        format!(
                            "There is no support yet for Str literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Bytes(_) => {
                    self.err(
                        format!(
                            "There is no support yet for Bytes literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Int(v) => {
                    let v_trunc  = v.to_u32_digits().1.into_iter().next().unwrap_or(0);
                    Ok(uint_lit(v_trunc, 32))
                }
                ast::Constant::Tuple(_) => {
                    self.err(
                        format!(
                            "There is no support yet for Tuple literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Float(_) => {
                    self.err(
                        format!(
                            "There is no support yet for Float literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Complex { real, imag } => {
                    self.err(
                        format!(
                            "There is no support yet for Complex literals.",
                        ),
                        &c.range()
                    )
                }
                ast::Constant::Ellipsis => {
                    self.err(
                        format!(
                            "There is no support yet for Ellipsis literals.",
                        ),
                        &c.range()
                    )
                }
            }
            Literal::Field(val, range) => Ok(field_lit(Integer::from_str_radix(val, 10).unwrap())),
        }
        .map_err(|err: String| format!("{err}"))
    }

    fn unary_op(&self, o: &ast::UnaryOp) -> fn(PyTerm) -> Result<PyTerm, String> {
        match o {
            // Invert in python is for signed ints (so need to mask in code)
            ast::UnaryOp::Invert => not,
            ast::UnaryOp::Not => not,
            ast::UnaryOp::UAdd => Ok,
            ast::UnaryOp::USub => neg,
        }
    }

    fn bin_op(&self, o: &ast::Operator) -> fn(PyTerm, PyTerm) -> Result<PyTerm, String> {
        match o {
            ast::Operator::Add => add,
            ast::Operator::Sub => sub,
            ast::Operator::Mult => mul,
            ast::Operator::MatMult => {
                unimplemented!("BinaryOperator {:#?} hasn't been implemented", o)
            }
            ast::Operator::Div => div,
            ast::Operator::Mod => rem,
            ast::Operator::Pow => pow,
            ast::Operator::LShift => shl,
            ast::Operator::RShift => shr,
            ast::Operator::BitOr => bitor,
            ast::Operator::BitXor => bitxor,
            ast::Operator::BitAnd => bitand,
            ast::Operator::FloorDiv => {
                unimplemented!("BinaryOperator {:#?} hasn't been implemented", o)
            }
        }
    }

    fn bool_op(&self, o: &ast::BoolOp) -> fn(PyTerm, PyTerm) -> Result<PyTerm, String> {
        match o {
            ast::BoolOp::And => and,
            ast::BoolOp::Or => or,
        }
    }

    fn cmp_op(&self, o: &ast::CmpOp) -> fn(PyTerm, PyTerm) -> Result<PyTerm, String> {
        match o {
            ast::CmpOp::Eq | ast::CmpOp::Is => eq,
            ast::CmpOp::NotEq | ast::CmpOp::IsNot => neq,
            ast::CmpOp::Lt => ult,
            ast::CmpOp::LtE => ule,
            ast::CmpOp::Gt => ugt,
            ast::CmpOp::GtE => uge,
            ast::CmpOp::In | ast::CmpOp::NotIn => {
                unimplemented!("ComparisonOperator {:?} hasn't been implemented", o);
            }
        }
    }

    fn cmp_ops(&self, o: &Vec<ast::CmpOp>) -> Vec<fn(PyTerm, PyTerm) -> Result<PyTerm, String>> {
        o
            .iter()
            .map(|op| self.cmp_op(op))
            .collect()
    }
    
    fn file_stack_push(&self, path: PathBuf) {
        self.file_stack.borrow_mut().push(path);
    }

    fn file_stack_pop(&self) -> Option<PathBuf> {
        self.file_stack.borrow_mut().pop()
    }

    fn file_stack_depth(&self) -> usize {
        self.file_stack.borrow().len()
    }

    fn function_ret_type(&self, s: &ast::StmtFunctionDef) -> Vec<ast::Expr> {
        match &s.returns {
            Some(s) => if let ast::Expr::Tuple(expr_tuple) = s.as_ref() {
                expr_tuple.elts.clone()
            } else {
                vec!(*s.clone())
            }
            None => panic!("Missing function return type.")
        }
    }
 
    fn function_param_type(&self, e: &ast::Arg) -> Result<ast::Expr, String> {
        match &e.annotation {
            Some(s) => Ok(*s.clone()),
            None => self.err(
                format!(
                    "Missing argument type.",
                ),
                &e.range()
            ),
        }
    }

    fn function_call_impl_<const IS_CNST: bool>(
        &self,
        args: Vec<PyTerm>,
        exp_ty: Option<Ty>,
        f_path: PathBuf,
        f_name: String,
    ) -> Result<PyTerm, String> {
        if IS_CNST {
            debug!("Const function call: {} {:?}", f_name, f_path);
        } else {
            debug!("Function call: {} {:?}", f_name, f_path);
        }

        let f = self
            .functions
            .get(&f_path)
            .ok_or_else(|| format!("No file '{:?}' attempting fn call", &f_path))?
            .get(&f_name)
            .ok_or_else(|| format!("No function '{}' attempting fn call", &f_name))?;

        let prev_func_call = self.curr_func.clone();
        self.curr_func.borrow_mut().replace_range(.., &f_name);

        let arg_tys = args.iter().map(|arg| arg.type_().clone());

        if self.stdlib.is_embed(&f_path) {
            Self::builtin_call(&f_name, args)
        } else {
            assert!(self.function_ret_type(&f).len() <= 1);
            if f.args.args.len() != args.len() {
                return Err(format!(
                    "Wrong number of arguments calling {} (got {}, expected {})",
                    &f.name.as_str(),
                    args.len(),
                    f.args.args.len()
                ));
            }

            let f = f.clone();
            self.file_stack_push(f_path);
            self.ret_ty_stack_push::<IS_CNST>(&f)?;

            // multi-return unimplemented
            let ret_ty = self.function_ret_type(&f)
                .first()
                .map(|r| self.type_impl_::<IS_CNST>(r))
                .transpose()?;
            let ret_ty = if IS_CNST {
                self.cvar_enter_function();
                ret_ty
            } else {
                self.circ_enter_fn(f_name, ret_ty);
                None
            };

            for (p, a) in f.args.args.into_iter().zip(args) {
                let ty = self.type_impl_::<IS_CNST>(&self.function_param_type(&p.def)?)?;
                if IS_CNST {
                    self.cvar_declare_init(p.def.arg.to_string(), &ty, a)?;
                } else {
                    self.circ_declare_init(p.def.arg.to_string(), ty, Val::Term(a))
                        .map_err(|e| format!("{e}"))?;
                }
            }

            for s in &f.body {
                self.stmt_impl_::<IS_CNST>(s)?;
            }

            let ret = if IS_CNST {
                self.cvar_exit_function();
                self.crets_pop()
            } else {
                self.circ_exit_fn()
                    .map(|a| a.unwrap_term())
                    .unwrap_or_else(|| py_bool_lit(false))
            };

            self.ret_ty_stack_pop();
            self.file_stack_pop();

            if IS_CNST {
                let ret_ty = ret_ty.unwrap_or(Ty::Bool);
                if ret.type_() != &ret_ty {
                    return Err(format!(
                        "Return type mismatch: expected {}, got {}",
                        ret_ty,
                        ret.type_()
                    ));
                }
            }

            self.curr_func.borrow_mut().replace_range(.., prev_func_call.borrow().as_str());
            self.maybe_garbage_collect();
            Ok(ret)
        }
    }

    fn maybe_garbage_collect(&self) {
        let est = self.gc_depth_estimate.get();
        let cur = self.file_stack_depth();
        if GC_INC * cur < est {
            if maybe_garbage_collect() {
                // we ran the GC and it did something; increase depth at which we run gc by 1 call
                self.gc_depth_estimate.set(est + GC_INC);
            } else {
                // otherwise, decrease depth at which we run gc by one call
                self.gc_depth_estimate.set(est.saturating_sub(GC_INC));
            }
        } else {
            // we didn't try to run the GC; just gradually increase the depth at which we'll run the gc
            let est_inc = (GC_INC * cur - est) / GC_INC;
            self.gc_depth_estimate.set(est + 1 + est_inc);
        }
    }

    fn const_entry_fn(&self, n: &str) -> PyTerm {
        debug!("Const entry: {}", n);
        let (f_file, f_name) = self.deref_import(n);
        if let Some(f) = self.functions.get(&f_file).and_then(|m| m.get(&f_name)) {
            if !f.args.args.is_empty() {
                panic!("const_entry_fn must be called on a function with zero arguments")
            }
        } else {
            panic!(
                "No function '{:?}//{}' attempting const_entry_fn",
                &f_file, &f_name
            );
        }

        self.function_call_impl_::<true>(Vec::new(), None, f_file, f_name)
            .unwrap_or_else(|e| panic!("const_entry_fn failed: {}", e))
    }

    fn entry_fn(&self, n: &str) {
        debug!("Entry: {}", n);
        // find the entry function
        let (f_file, f_name) = self.deref_import(n);
        let prev_func_call = self.curr_func.clone();
        self.curr_func.borrow_mut().replace_range(.., &f_name);
        let f = self
            .functions
            .get(&f_file)
            .unwrap_or_else(|| panic!("No file '{:?}'", &f_file))
            .get(&f_name)
            .unwrap_or_else(|| panic!("No function '{}'", &f_name))
            .clone();
        // tuple returns not supported
        assert!(self.function_ret_type(&f).len() <= 1);
        // get return type
        let ret_ty = self.function_ret_type(&f).first().map(|r| self.type_(r));
        // set up stack frame for entry function
        self.circ_enter_fn(n.to_owned(), ret_ty.clone());
        let mut persistent_arrays: Vec<String> = Vec::new();
        for p in f.args.args.iter() {
            let ty = self.type_(&self.function_param_type(&p.def).unwrap_or_else(|a| panic!("{a}")));
            debug!("Entry param: {}: {}", p.def.arg.as_str(), ty);
            let vis = self.interpret_visibility(&p.def);
            if let PyVis::Committed = &vis {
                persistent_arrays.push(p.def.arg.to_string());
            }
            let r = self.circ_declare_input(p.def.arg.to_string(), &ty, vis, None, false);
            self.unwrap(r, &p.def.range());
        }
        for s in &f.body {
            self.unwrap(self.stmt_impl_::<false>(s), &s.range());
        }
        for a in persistent_arrays {
            let term = self
                .circ_get_value(Loc::local(a.clone()))
                .unwrap()
                .unwrap_term()
                .term;
            trace!("End persistent_array {a}, {}", term);
            self.circ.borrow_mut().end_persistent_array(&a, term);
        }
        if let Some(r) = self.circ_exit_fn() {
            match self.mode {
                Mode::Mpc(_) => {
                    let ret_term = r.unwrap_term();
                    let ret_terms = ret_term.terms();
                    self.circ
                        .borrow()
                        .cir_ctx()
                        .cs
                        .borrow_mut()
                        .outputs
                        .extend(ret_terms);
                }
                Mode::Proof => {
                    let ty = ret_ty.as_ref().unwrap();
                    let name = "return".to_owned();
                    let ret_val = r.unwrap_term();
                    let ret_var_val = self
                        .circ_declare_input(name, ty, PyVis::Public, Some(ret_val.clone()), false)
                        .expect("circ_declare return");
                    let ret_eq = eq(ret_val, ret_var_val).unwrap().term;
                    let mut assertions = std::mem::take(&mut *self.assertions.borrow_mut());
                    let to_assert = if assertions.is_empty() {
                        ret_eq
                    } else {
                        assertions.push(ret_eq);
                        term(AND, assertions)
                    };
                    debug!("Assertion: {}", to_assert);
                    self.circ.borrow_mut().assert(to_assert);
                }
                Mode::Opt => {
                    let ret_term = r.unwrap_term();
                    let ret_terms = ret_term.terms();
                    assert!(
                        ret_terms.len() == 1,
                        "When compiling to optimize, there can only be one output"
                    );
                    let t = ret_terms.into_iter().next().unwrap();
                    let t_sort = check(&t);
                    if !matches!(t_sort, Sort::BitVector(_)) {
                        panic!("Cannot maximize output of type {}", t_sort);
                    }
                    self.circ.borrow().cir_ctx().cs.borrow_mut().outputs.push(t);
                }
                Mode::ProofOfHighValue(v) => {
                    let ret_term = r.unwrap_term();
                    let ret_terms = ret_term.terms();
                    assert!(
                        ret_terms.len() == 1,
                        "When compiling to optimize, there can only be one output"
                    );
                    let t = ret_terms.into_iter().next().unwrap();
                    let cmp = match check(&t) {
                        Sort::BitVector(w) => term![BV_UGE; t, bv_lit(v, w)],
                        s => panic!("Cannot maximize output of type {}", s),
                    };
                    self.circ
                        .borrow()
                        .cir_ctx()
                        .cs
                        .borrow_mut()
                        .outputs
                        .push(cmp);
                }
            }
        }
        self.curr_func.borrow_mut().replace_range(.., prev_func_call.borrow().as_str());
    }

    fn interpret_visibility(&self, arg: &ast::Arg) -> PyVis {
        match *arg.annotation.clone().unwrap() {
            ast::Expr::Subscript(e) => if let ast::Expr::Name(n) = *e.clone().value {
                // in ZoKrates it assumes None is public, however
                // we have to be strict since we match against an
                // annotation, which is a string.
                if n.id.as_str() == "Private" {
                    PyVis::Private(PROVER_ID)
                } else if n.id.as_str() == "Committed" {
                    PyVis::Committed
                } else if n.id.as_str() == "Public" {
                    PyVis::Public
                } else {
                    self.err(
                        format!(
                            "Incorrect visibility specifier used",
                        ),
                        &arg.range()
                    );
                }
            } else {
                // if no name is found
                self.err(
                    format!(
                        "Incorrect visibility specifier used",
                    ),
                    &arg.range()
                );
            }
            // if not otherwise specified, return error
            _ => self.err(
                format!(
                    "Incorrect visibility specifier used",
                ),
                &arg.range()
            )
        }
    }

    fn cur_path(&self) -> PathBuf {
        self.file_stack.borrow().last().unwrap().to_path_buf()
    }

    fn cur_dir(&self) -> PathBuf {
        let mut p = self.cur_path();
        p.pop();
        p
    }

    fn cur_import_map(&self) -> Option<&HashMap<String, (PathBuf, String)>> {
        self.import_map
            .get(self.file_stack.borrow().last().unwrap())
    }

    fn deref_import(&self, s: &str) -> (PathBuf, String) {
        // import map is flattened, so we only need to chase through at most one indirection
        self.cur_import_map()
            .and_then(|m| m.get(s))
            .cloned()
            .unwrap_or_else(|| (self.cur_path(), s.to_string()))
    }

    fn const_ty_lookup_(&self, i: &str) -> Option<&ast::Expr> {
        let (f_file, f_name) = self.deref_import(i);
        self.constants
            .get(&f_file)
            .and_then(|m| m.get(&f_name))
            .map(|(t, _)| t)
    }

    fn const_lookup_(&self, i: &str) -> Option<&PyTerm> {
        let (f_file, f_name) = self.deref_import(i);
        self.constants
            .get(&f_file)
            .and_then(|m| m.get(&f_name))
            .map(|(_, v)| v)
    }

    fn const_defined(&self, i: &str) -> bool {
        let (f_file, f_name) = self.deref_import(i);
        self.constants
            .get(&f_file)
            .map(|m| m.contains_key(&f_name))
            .unwrap_or(false)
    }

    fn identifier_impl_<const IS_CNST: bool>(
        &self,
        i: &ast::ExprName,
    ) -> Result<PyTerm, String> {
        match self.const_lookup_(&i.id.as_str()).cloned() {
            Some(v) => Ok(v),
            None if IS_CNST => self.cvar_lookup(&i.id.as_str()).ok_or_else(|| {
                format!(
                    "Undefined const identifier {} in {}",
                    &i.id.as_str(),
                    self.cur_path().canonicalize().unwrap().to_string_lossy()
                )
            }),
            _ => match self
                .circ_get_value(Loc::local(i.id.to_string()))
                .map_err(|e| format!("{e}"))?
            {
                Val::Term(t) => Ok(t),
                _ => Err(format!("Non-Term identifier {}", &i.id.as_str())),
            },
        }
    }

    fn const_isize_impl_<const IS_CNST: bool>(
        &self,
        e: &ast::Expr,
    ) -> Result<isize, String> {
        const_int(self.expr_impl_::<IS_CNST>(e)?)?
            .to_isize()
            .ok_or_else(|| "Constant integer outside isize range".to_string())
    }

    fn const_usize_impl_<const IS_CNST: bool>(
        &self,
        e: &ast::Expr,
    ) -> Result<usize, String> {
        const_int(self.expr_impl_::<IS_CNST>(e)?)?
            .to_usize()
            .ok_or_else(|| "Constant integer outside usize range".to_string())
    }

    fn const_usize_(&self, e: &ast::Expr) -> Result<usize, String> {
        self.const_usize_impl_::<true>(e)
    }

    fn array_access_impl_<const IS_CNST: bool>(
        &self,
        target: &ast::ExprSubscript,
        val: PyTerm,
    ) -> Result<PyTerm, String> {
        let array_size = if let Ty::Array(s, _) = val.ty {
            s
        } else {
            0
        };
        match target.slice.as_ref() {
            ast::Expr::Slice(r) => {
                let s = r
                    .lower
                    .as_ref()
                    .map(|s| self.const_usize_impl_::<IS_CNST>(&s))
                    .transpose()?
                    .map(|value| if value <= array_size { value } else { array_size - (u32::MAX as usize - value + 1) });
                let e = r
                    .upper
                    .as_ref()
                    .map(|s| self.const_usize_impl_::<IS_CNST>(&s))
                    .transpose()?
                    .map(|value| if value <= array_size { value } else { array_size - (u32::MAX as usize - value + 1) });
                let step = r
                    .step
                    .as_ref()
                    .map(|s| self.const_usize_impl_::<IS_CNST>(&s))
                    .transpose()?
                    .map(|value| if value <= (u32::MAX / 2) as usize {value as isize} else {(u32::MAX as usize - value + 1) as isize * (-1)});
                slice(val, s, e, step)
            }
            _ => {
                array_select(val, self.expr_impl_::<IS_CNST>(&target.slice)?)
            }
        }
    }

    fn expr_impl_<const IS_CNST: bool>(&self, e: &ast::Expr) -> Result<PyTerm, String> {
        if IS_CNST {
            debug!("Const expr range: {:?}", e.range());
        } else {
            debug!("Expr range: {:?}", e.range());
        }

        match e {
            ast::Expr::BoolOp(b) => {
                let values: Vec<PyTerm> = b.values.clone()
                    .iter()
                    .map(|v| self.expr_impl_::<IS_CNST>(&v))
                    .collect::<Result<Vec<_>, _>>()?;
                let op = self.bool_op(&b.op);
                let (init_result, rest) = values.split_first().unwrap();
                let final_result = rest.iter().try_fold(init_result.clone(), |prev_result, term| {
                    op(prev_result.clone(), term.clone())
                });
                final_result
            }
            ast::Expr::NamedExpr(n) => {
                // This is for the famous walrus operator `:=`
                self.err(
                    format!("Named expressions are not supported yet."),
                    &n.range(),
                )
            }
            ast::Expr::BinOp(b) => {
                let left = self.expr_impl_::<IS_CNST>(&b.left)?;
                let right = self.expr_impl_::<IS_CNST>(&b.right)?;
                let op = self.bin_op(&b.op);
                op(left, right)
            }
            ast::Expr::UnaryOp(u) => {
                let arg = self.expr_impl_::<IS_CNST>(&u.operand)?;
                let op = self.unary_op(&u.op);
                op(arg)
            }
            ast::Expr::Lambda(l) => {
                self.err(
                    format!("Lambda expressions are not supported yet."),
                    &l.range(),
                )
            }
            ast::Expr::IfExp(u) => {
                match self.expr_impl_::<true>(&u.test).ok().and_then(const_bool) {
                    Some(true) => self.expr_impl_::<IS_CNST>(&u.body),
                    Some(false) => self.expr_impl_::<IS_CNST>(&u.orelse),
                    None if IS_CNST => Err("ternary condition not const bool".to_string()),
                    _ => {
                        let c = self.expr_impl_::<false>(&u.test)?;
                        let cbool = bool(c.clone())?;
                        self.circ_enter_condition(cbool.clone());
                        let a = self.expr_impl_::<false>(&u.body)?;
                        self.circ_exit_condition();
                        self.circ_enter_condition(term![NOT; cbool]);
                        let b = self.expr_impl_::<false>(&u.orelse)?;
                        self.circ_exit_condition();
                        cond(c, a, b)
                    }
                }
            }
            ast::Expr::Dict(d) => {
                self.err(
                    format!("Dicts are not supported yet."),
                    &d.range(),
                )
            }
            ast::Expr::Set(s) => {
                self.err(
                    format!("Sets are not supported yet."),
                    &s.range(),
                )
            }
            ast::Expr::ListComp(lc) => {
                // For now we only allow list comprehension to be used for
                // array initialization of constant values (or other arrays).
                // In the future it would be nice to support other expressions
                // and loop over custom iterators.

                // Also, get rid of code-reuse
                match lc.elt.as_ref() {
                    ast::Expr::ListComp(lcc) => {
                        let val = self.expr_impl_::<IS_CNST>(&ast::Expr::from(lcc.clone()))?;
                        // in the future handle generators correctly
                        if let ast::Expr::Call(cc) = &lc.generators[0].iter {
                            if cc.args.len() == 1 {
                                let num = self.const_usize_impl_::<IS_CNST>(&cc.args[0])?;
                                fill_array(val, num)
                            } else {
                                self.err(
                                    format!("Range takes at most 1 element (for now)."),
                                    &cc.range(),
                                )
                            }
                        } else {
                            self.err(
                                format!("Range is missing."),
                                &lcc.range(),
                            )
                        }
                    }
                    _ => {
                        let val = self.expr_impl_::<IS_CNST>(&lc.elt)?;
                        // in the future handle generators correctly
                        if let ast::Expr::Call(cc) = &lc.generators[0].iter {
                            if cc.args.len() == 1 {
                                let num = self.const_usize_impl_::<IS_CNST>(&cc.args[0])?;
                                fill_array(val, num)
                            } else {
                                self.err(
                                    format!("Range takes at most 1 element (for now)."),
                                    &cc.range(),
                                )
                            }
                        } else {
                            self.err(
                                format!("Range is missing."),
                                &lc.elt.range(),
                            )
                        }
                    }
                }
            }
            ast::Expr::SetComp(sc) => {
                self.err(
                    format!("Set comprehension is not supported yet."),
                    &sc.range(),
                )
            }
            ast::Expr::DictComp(dc) => {
                self.err(
                    format!("Dictionary comprehension is not supported yet."),
                    &dc.range(),
                )
            }
            ast::Expr::GeneratorExp(g) => {
                // in theory we could support this easily if we defined type
                // casting into list, but not a priority for now.
                self.err(
                    format!("Generators are not supported yet."),
                    &g.range(),
                )
            }
            ast::Expr::Await(a) => {
                self.err(
                    format!("Await expressions are not supported yet."),
                    &a.range(),
                )
            }
            ast::Expr::Yield(y) => {
                self.err(
                    format!("Yield expressions are not supported yet."),
                    &y.range(),
                )
            }
            ast::Expr::YieldFrom(yf) => {
                self.err(
                    format!("Yield from expressions are not supported yet."),
                    &yf.range(),
                )
            }
            ast::Expr::Compare(b) => {
                // there is probably a nicer and more efficient way to do this
                let mut comparators: Vec<PyTerm> = vec![self.expr_impl_::<IS_CNST>(&b.left)?];
                let comparators_rest: Vec<PyTerm> = b.comparators.clone()
                    .iter()
                    .map(|e| self.expr_impl_::<IS_CNST>(&e))
                    .collect::<Result<Vec<_>, _>>()?;
                comparators.extend(comparators_rest);
                let ops = self.cmp_ops(&b.ops);
                let results_intm: Vec<PyTerm> = comparators
                    .windows(2)
                    .zip(ops.iter())
                    .map(|(pair, f)| f(pair[0].clone(), pair[1].clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                let (init_result, rest) = results_intm.split_first().unwrap();
                let final_result = rest.iter().try_fold(init_result.clone(), |prev_result, term| {
                    and(prev_result.clone(), term.clone())
                });
                final_result
            }
            ast::Expr::Call(p) => {
                // Note that args and kwargs are used in function calls
                // and class instantiation respectively (and not interchangably).
                // This may be improved in the future.
                let (f_path, f_name) = if let ast::Expr::Name(n) = p.func.as_ref() {
                    self.deref_import(n.id.as_str())
                } else {
                    (PathBuf::new(), String::new())
                };
                let exp_ty = self.lhs_ty_take().and_then(|ty| Some(ty));
                // if p.args.is_empty() && p.keywords.is_empty() {
                //     self.err(
                //         format!("Callable requires either arguments or keywords."),
                //         &p.range(),
                //     )
                // }
                // This is ugly but necessary for now:
                // Since fields are not natively supported, we both check if f_name == "field"
                // and if the argument is ast::Expr::Constant. We do this instead of typecasting from
                // int literal to avoid wrapping around.
                let args = p
                    .args
                    .iter()
                    .map(|e| match e {
                        ast::Expr::Constant(c) if f_name == "field" => {
                            let val = match c.value.as_int() {
                                Some(i) => i.to_string(),
                                None => self.err(
                                    format!("Constant expected inside of field literal declaration."),
                                    &c.range(),
                                ),
                            };
                            self.literal_(&Literal::Field(val, c.range()))
                        }
                        _ => self.expr_impl_::<IS_CNST>(e),
                    })
                    .collect::<Result<Vec<_>,_>>()?;
                let kwargs = p
                    .keywords
                    .iter()
                    .map(|m| {
                        self.expr_impl_::<IS_CNST>(&m.value)
                            .map(|m_expr| (m.arg.clone().unwrap().to_string(), m_expr))
                    })
                    .collect::<Result<Vec<_>, String>>();
                // As usual, there is probably a nicer way to do this:
                // Handle "type casting" arguments as respective literals
                // Also handle class object creation appropriately
                // and ignore some keywords like "assert" (or at least handle them differently)
                if f_name == "int" {
                    // Unclean explicit type casting, refactor later.
                    if args[0].ty == Ty::Field && args.len() == 1 {
                        uint_from_bits(field_to_bits(args[0].clone(), 32)?)
                    } else if args[0].ty == Ty::Uint(32) && args.len() == 1 {
                        Ok(args[0].clone())
                    } else if args[0].ty == Ty::Bool && args.len() == 1 {
                        uint_from_bool(args[0].clone(), 32)
                    } else if args.len() != 1 {
                        self.err(
                            format!("Int takes at most 1 argument."),
                            &p.range(),
                        )
                    } else {
                        self.err(
                            format!("Type casting into int is only possible from a field, int or bool."),
                            &p.range(),
                        )
                    }
                } else if f_name == "float" {
                    self.err(
                        format!("Floats are not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "complex" {
                    self.err(
                        format!("Complex numbers are not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "str" {
                    self.err(
                        format!("Strings are not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "ord" {
                    self.err(
                        format!("Type casting into int is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "hex" {
                    self.err(
                        format!("Type casting into hexadecimal string is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "oct" {
                    self.err(
                        format!("Type casting into octal string is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "tuple" {
                    self.err(
                        format!("Type casting into tuple is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "set" {
                    self.err(
                        format!("Type casting into set is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "frozenset" {
                    self.err(
                        format!("Type casting into frozenset is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "list" {
                    self.err(
                        format!("Type casting into list is not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "dict" {
                    self.err(
                        format!("Dicts are not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "bool" {
                    // Unclean explicit type casting, refactor this later.
                    if args[0].ty == Ty::Bool && args.len() == 1 {
                        Ok(args[0].clone())
                    } else if args[0].ty == Ty::Uint(32) && args.len() == 1 {
                        neq(args[0].clone(), uint_lit(0, 32))
                    } else if args[0].ty == Ty::Field && args.len() == 1 {
                        neq(args[0].clone(), field_lit(0))
                    } else if args.len() != 1 {
                        self.err(
                            format!("Bool takes at most 1 argument."),
                            &p.range(),
                        )
                    } else {
                        self.err(
                            format!("Type casting into bool is only possible from a field, int or bool."),
                            &p.range(),
                        )
                    }
                } else if f_name == "range" {
                    self.err(
                        format!("Iterators are not supported yet."),
                        &p.range(),
                    )
                } else if f_name == "field" {
                    // Unclean explicit type casting, refactor later.
                    if args[0].ty == Ty::Uint(32) && args.len() == 1 {
                        uint_to_field(args[0].clone())
                    } else if args[0].ty == Ty::Field && args.len() == 1 {
                        Ok(args[0].clone())
                    } else if args[0].ty == Ty::Bool && args.len() == 1 {
                        uint_to_field(uint_from_bool(args[0].clone(), 32)?)
                    } else if args.len() != 1 {
                        self.err(
                            format!("Field takes at most 1 argument."),
                            &p.range(),
                        )
                    } else {
                        self.err(
                            format!("Type casting into field is only possible from a field, int or bool."),
                            &p.range(),
                        )
                    }
                } else if let Some(m) = self.classes_and_tys.get(&f_path) {
                    // If f_name is a defined class, 
                    if m.contains_key(&f_name) {
                        return Ok(PyTerm::new_class(self.canon_class(&f_name)?, kwargs.unwrap()))
                    } else {
                        self.function_call_impl_::<IS_CNST>(args, exp_ty, f_path, f_name)
                    }
                    // I'm too lazy to avoid code repetition atm.
                } else {
                    self.function_call_impl_::<IS_CNST>(args, exp_ty, f_path, f_name)
                }
            }
            ast::Expr::FormattedValue(fv) => {
                // We would need to support joined strings
                self.err(
                    format!("Formatted values are not supported yet."),
                    &fv.range(),
                )
            }
            ast::Expr::JoinedStr(js) => {
                // We would need to support strings in the first place
                self.err(
                    format!("Joined strings are not supported yet."),
                    &js.range(),
                )
            }
            ast::Expr::Constant(c) => self.literal_(&Literal::PyLiteral(c.clone())),
            ast::Expr::Attribute(a) => {
                match a.value.as_ref() {
                    ast::Expr::Attribute(aa) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(aa.clone()));
                        field_select(&v?, &a.attr.as_str())
                    }
                    ast::Expr::Subscript(s) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(s.clone()));
                        field_select(&v?, &a.attr.as_str())
                    }
                    ast::Expr::Name(n) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(n.clone()));
                        field_select(&v?, &a.attr.as_str())
                    }
                    e => {
                        self.err(
                            format!("Attribute or subscript must be associated to an identifier, another attribute or another subscript."),
                            &e.range(),
                        )
                    }
                }
            }
            ast::Expr::Subscript(s) => {
                match s.value.as_ref() {
                    ast::Expr::Attribute(a) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(a.clone()));
                        self.array_access_impl_::<IS_CNST>(s, v?)
                    }
                    ast::Expr::Subscript(ss) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(ss.clone()));
                        self.array_access_impl_::<IS_CNST>(s, v?)
                    }
                    ast::Expr::Name(n) => {
                        let v = self.expr_impl_::<IS_CNST>(&ast::Expr::from(n.clone()));
                        self.array_access_impl_::<IS_CNST>(s, v?)
                    }
                    err => {
                        self.err(
                            format!("Attribute or subscript must be associated to an identifier, another attribute or another subscript."),
                            &err.range(),
                        )
                    }
                }
            }
            ast::Expr::Starred(s) => {
                // They are generally used for destructuring,
                // but we only support it for array spreads
                // to copy values of array
                self.err(
                    format!("Starred expressions are only supported inside of inline arrays."),
                    &s.range(),
                )
            }
            ast::Expr::Name(i) => self.identifier_impl_::<IS_CNST>(i),
            ast::Expr::List(l) => {
                let mut avals = Vec::with_capacity(l.elts.len());
                l.elts
                    .iter()
                    .try_for_each::<_, Result<_, String>>(|ee| match ee {
                        ast::Expr::Starred(s) => {
                            avals.append(
                                &mut self.expr_impl_::<IS_CNST>(&s.value)?.unwrap_array()?,
                            );
                            Ok(())                        }
                        _ => {
                            avals.push(self.expr_impl_::<IS_CNST>(ee)?);
                            Ok(())
                        }
                    })?;
                PyTerm::new_array(avals)
            }
            ast::Expr::Tuple(t) => {
                // This should be relatively straight forward to implement
                self.err(
                    format!("Tuples are not supported yet."),
                    &t.range(),
                )
            }
            ast::Expr::Slice(s) => {
                // This is already being handled by array_access_impl.
                // Maybe refactor in the future.
                self.err(
                    format!("If we reached the slice expression then something went wrong..."),
                    &s.range(),
                )
            }
        }
        .and_then(|res| if IS_CNST { const_val(res) } else { Ok(res) })
        .map_err(|err| format!("{err}"))
    }

    fn canon_class(&self, id: &str) -> Result<String, String> {
        match self
            .get_class_or_type(id)
            .ok_or_else(|| format!("No such class or type {id} canonicalizing class creation"))?
            .0
        {
            Ok(_) => Ok(id.to_string()),
            // Technically this is only supported in python >=3.12
            // But the rustpython parser already supports it
            Err(t) => match t.value.as_ref() {
                ast::Expr::Name(s) => self.canon_class(&s.id.as_str()),
                _ => Err(format!("Found non-class canonicalizing class {id}")),
            },
        }
    }

    fn ret_impl_<const IS_CNST: bool>(&self, ret: Option<PyTerm>) -> Result<(), CircError> {
        if IS_CNST {
            self.crets_push(ret.unwrap_or_else(|| py_bool_lit(false)));
            Ok(())
        } else {
            self.circ_return_(ret)
        }
    }

    fn decl_impl_<const IS_CNST: bool>(&self, name: String, ty: &Ty) -> Result<(), String> {
        if IS_CNST {
            self.cvar_declare(name, ty)
        } else {
            self.circ
                .borrow_mut()
                .declare_uninit(name, ty)
                .map_err(|e| format!("{e}"))
        }
    }

    fn declare_init_impl_<const IS_CNST: bool>(
        &self,
        name: String,
        ty: Ty,
        val: PyTerm,
    ) -> Result<(), String> {
        if IS_CNST {
            self.cvar_declare_init(name, &ty, val)
        } else {
            self.circ_declare_init(name, ty, Val::Term(val))
                .map(|_| ())
                .map_err(|e| format!("{e}"))
        }
    }

    fn stmt_impl_<const IS_CNST: bool>(&self, s: &ast::Stmt) -> Result<(), String> {
        if IS_CNST {
            debug!("Const expr range: {:?}", s.range());
        } else {
            debug!("Stmt range: {:?}", s.range());
        }

        match s {
            // FunctionDef, ClassDef and Import(From) are handled in visit_body()
            ast::Stmt::FunctionDef(f) => {
                self.err(
                    format!("Function def statements are only supported in the module body."),
                    &f.range(),
                )
            }
            ast::Stmt::AsyncFunctionDef(a) => {
                self.err(
                    format!("Aync function definitions are not supported yet."),
                    &a.range(),
                )
            }
            ast::Stmt::ClassDef(c) => {
                self.err(
                    format!("Class def statements are only supported in the module body."),
                    &c.range(),
                )
            }
            ast::Stmt::Return(r) => {
                // Multi-return is not implemented
                if let Some(e) = r.value.as_ref() {
                    self.set_lhs_ty_ret(r);
                    let ret = self.expr_impl_::<IS_CNST>(&e)?;
                    self.ret_impl_::<IS_CNST>(Some(ret))
                } else {
                    self.ret_impl_::<IS_CNST>(None)
                }
                .map_err(|e| format!("{e}"))
            }
            ast::Stmt::Delete(d) => {
                self.err(
                    format!("Delete statements are not supported yet."),
                    &d.range(),
                )
            }
            ast::Stmt::Assign(a) => {
                // Multi-assignment is not implemented
                assert!(a.targets.len() <= 1);

                // We might not need this if we have no generics
                self.set_lhs_ty_defn::<IS_CNST>(a.value.as_ref(), &ast::Stmt::from(a.clone()))?;
                let e = self.expr_impl_::<IS_CNST>(&a.value)?;
                let name = self.get_lhs_name::<IS_CNST>(&a.targets[0]).unwrap();
                // For now we pass strictness condition as false because
                // it is not clear how this would apply in Python.
                self.assign_impl_::<IS_CNST>(&name.id.as_str(), Some(a.targets[0].clone()), e, false)
            }
            ast::Stmt::TypeAlias(t) => {
                self.err(
                    format!("Type aliases are not supported yet."),
                    &t.range(),
                )
            }
            ast::Stmt::AugAssign(a) => {
                // Just wrap this into target = target + value assignment

                // We might not need this if we have no generics
                self.set_lhs_ty_defn::<IS_CNST>(a.value.as_ref(), &ast::Stmt::from(a.clone()))?;
                let left = self.expr_impl_::<IS_CNST>(&a.target)?;
                let right = self.expr_impl_::<IS_CNST>(&a.value)?;
                let op = self.bin_op(&a.op);
                let e = op(left, right)?;
                let name = self.get_lhs_name::<IS_CNST>(&a.target).unwrap();
                // For now we pass strictness condition as false because
                // it is not clear how this would apply in Python.
                self.assign_impl_::<IS_CNST>(&name.id.as_str(), Some(*a.target.clone()), e, false)
            }
            ast::Stmt::AnnAssign(a) => {
                // We might not need this if we have no generics
                self.set_lhs_ty_defn::<IS_CNST>(a.value.as_ref().unwrap(), &ast::Stmt::from(a.clone()))?;
                let e = self.expr_impl_::<IS_CNST>(&a.value.as_ref().unwrap())?;
                let name = self.get_lhs_name::<IS_CNST>(&a.target).unwrap();
                let decl_ty = self.type_impl_::<IS_CNST>(&a.annotation)?;
                let ty = e.type_();
                if &decl_ty != ty {
                    return Err(format!(
                        "Assignment type mismatch: {decl_ty} annotated vs {ty} actual",
                    ));
                }
                self.declare_init_impl_::<IS_CNST>(
                    name.id.to_string(),
                    decl_ty,
                    e,
                )
            }
            ast::Stmt::For(i) => {
                // Assume that iteration variable is always u32 (necessary for now)
                let ty = Ty::Uint(32);
                let ival_cons = PyTerm::new_u32::<isize>;

                let ast::Expr::Call(c) = i.iter.as_ref() else {
                    self.err(
                        format!("For loop range is missing."),
                        &i.iter.range(),
                    )
                };

                let ast::Expr::Name(n) = i.target.as_ref() else {
                    self.err(
                        format!("Missing iteration variable in for loop."),
                        &i.iter.range(),
                    )
                };

                let mut s: isize = 0;
                let mut e: isize = 0;
                // At some point it would be nice to be able to
                // handle step sizes as well.
                if c.args.len() == 1 {
                    s = 0;
                    // not sure if this works, if not then use commented part

                    e = self.const_isize_impl_::<IS_CNST>(&c.args[0])?;
                    // if let ast::Expr::Constant(cc) = c.args[0] {
                    //     let e = self.const_isize_impl_::<IS_CNST>(&ast::Expr::from(cc.clone()))?;
                    // } else {
                    //     self.err(
                    //         format!("For loop range must be statically bound."),
                    //         &c.range(),
                    //     )
                    // }
                } else if c.args.len() == 2 {
                    // same argument as above

                   s = self.const_isize_impl_::<IS_CNST>(&c.args[0])?;
                    // if let ast::Expr::Constant(cc) = c.args[0] {
                    //     let s: isize = self.const_isize_impl_::<IS_CNST>(&ast::Expr::from(cc.clone()))?;
                    // } else {
                    //     self.err(
                    //         format!("For loop range must be statically bound."),
                    //         &c.range(),
                    //     )
                    // }
                    e = self.const_isize_impl_::<IS_CNST>(&c.args[1])?;
                    // if let ast::Expr::Constant(cc) = c.args[1] {
                    //     let e = self.const_isize_impl_::<IS_CNST>(&ast::Expr::from(cc.clone()))?;
                    // } else {
                    //     self.err(
                    //         format!("For loop range must be statically bound."),
                    //         &c.range(),
                    //     )
                    // }
                } else {
                    self.err(
                        format!("For loop range requires at least 1 and at most 2 arguments."),
                        &c.range(),
                    )
                }

                let v_name = n.id.to_string();
                self.enter_scope_impl_::<IS_CNST>();
                self.decl_impl_::<IS_CNST>(v_name, &ty)?;
                for j in s..e {
                    self.enter_scope_impl_::<IS_CNST>();
                    self.assign_impl_::<IS_CNST>(&n.id.as_str(), None, ival_cons(j), false)?;
                    for s in &i.body {
                        self.stmt_impl_::<IS_CNST>(s)?;
                    }
                    self.exit_scope_impl_::<IS_CNST>();
                }
                self.exit_scope_impl_::<IS_CNST>();
                Ok(())
            }
            ast::Stmt::AsyncFor(a) => {
                self.err(
                    format!("Aync for statements are not supported yet."),
                    &a.range(),
                )
            }
            ast::Stmt::While(w) => {
                // I think this could be done if we can infer what the iteration variable
                // is inside of the while loop.
                self.err(
                    format!("While statements are not supported yet."),
                    &w.range(),
                )    
            }
            ast::Stmt::If(i) => {
                // We could implement this as a wrapper for the ternary
                // operator, but at the moment this is not worth it.
                self.err(
                    format!("If statements are not supported yet. Use the ternary operator."),
                    &i.range(),
                )   
            }
            ast::Stmt::With(w) => {
                self.err(
                    format!("With statements are not supported yet."),
                    &w.range(),
                )
            }
            ast::Stmt::AsyncWith(a) => {
                self.err(
                    format!("Aync with statements definitions are not supported yet."),
                    &a.range(),
                )
            }
            ast::Stmt::Match(m) => {
                self.err(
                    format!("Match statements are not supported yet."),
                    &m.range(),
                )
            }
            ast::Stmt::Raise(r) => {
                self.err(
                    format!("Raise statements are not supported yet."),
                    &r.range(),
                )
            }
            ast::Stmt::Try(t) => {
                self.err(
                    format!("Try statements are not supported yet."),
                    &t.range(),
                )
            }
            ast::Stmt::TryStar(t) => {
                // This is new syntax from python 3.11
                self.err(
                    format!("Try star statements are not supported yet."),
                    &t.range(),
                )
            }
            ast::Stmt::Assert(e) => {
                match self.expr_impl_::<true>(&e.test).and_then(|v| {
                    const_bool(v)
                        .ok_or_else(|| "interpreting expr as const bool failed".to_string())
                }) {
                    Ok(true) => Ok(()),
                    Ok(false) => Err(format!(
                        "Const assert failed: {} at\n{:?}",
                        e.msg
                            .as_ref()
                            .map(|m| if let ast::Expr::Constant(msg) = m.as_ref() {
                                msg.value.as_str().unwrap()
                            } else {
                                "(no error message given)"
                            })
                            .unwrap(),
                        e.test.range()
                    )),
                    Err(err) if IS_CNST => Err(format!(
                        "Const assert expression eval failed {} at\n{:?}",
                        err,
                        e.test.range(),
                    )),
                    _ => {
                        let b = bool(self.expr_impl_::<false>(&e.test)?)?;
                        self.assert(b);
                        Ok(())
                    }
                }
            }
            ast::Stmt::Import(i) => {
                self.err(
                    format!("Import statements are only supported in the module body."),
                    &i.range(),
                )
            }
            ast::Stmt::ImportFrom(i) => {
                self.err(
                    format!("Import from statements are only supported in the module body."),
                    &i.range(),
                )
            }
            ast::Stmt::Global(g) => {
                // I may use this to declare constant variables
                self.err(
                    format!("Global statements are not supported yet."),
                    &g.range(),
                )
            }
            ast::Stmt::Nonlocal(n) => {
                self.err(
                    format!("Non-local statements are not supported yet."),
                    &n.range(),
                )
            }
            ast::Stmt::Expr(_) => {
                // We can just escape it, since it does not contribute to the circuit
                Ok(())
            }
            ast::Stmt::Pass(p) => {
                // We could probably just escape this and then check
                // in FunctionDef whether this is or return is passed,
                // and interpret accordingly.
                self.err(
                    format!("Pass statements are not supported yet."),
                    &p.range(),
                )
            }
            ast::Stmt::Break(b) => {
                // This would not be possible because it is the result
                // of a condition, and would require branching without
                // assignment or return of same type
                self.err(
                    format!("Break statements are not supported yet."),
                    &b.range(),
                )
            }
            ast::Stmt::Continue(c) => {
                // The issue is that branching requires assignment or return
                self.err(
                    format!("Continue statements are not supported yet."),
                    &c.range(),
                )
            }
        }
        .map_err(|err| format!("{err}"))
    }

    fn set_lhs_ty_defn<const IS_CNST: bool>(
        &self,
        val: &ast::Expr,
        d: &ast::Stmt
    ) -> Result<(), String> {
        assert!(self.lhs_ty.borrow().is_none());
        if matches!(val.clone(), ast::Expr::Call(_)) {
            let ty = Some(self.lhs_type::<IS_CNST>(
                d)
            ).transpose()?;
            self.lhs_ty_put(ty);
        }
        Ok(())
    }

    fn set_lhs_ty_ret(&self, r: &ast::StmtReturn) {
        assert!(self.lhs_ty.borrow().is_none());
        if matches!(*r.clone().value.unwrap(), ast::Expr::Call(_)) {
            let ty = self.ret_ty_stack_last();
            self.lhs_ty_put(ty);
        }
    }

    fn get_lhs_name<const IS_CNST: bool>(&self, e: &ast::Expr) -> Result<ast::ExprName, String> {
        match e {
            ast::Expr::Attribute(a) => {
                match a.value.as_ref() {
                    ast::Expr::Attribute(aa) => self.get_lhs_name::<IS_CNST>(&ast::Expr::from(aa.clone())),
                    ast::Expr::Subscript(s) => self.get_lhs_name::<IS_CNST>(&ast::Expr::from(s.clone())),
                    ast::Expr::Name(n) => Ok(n.clone()),
                    err => {
                        self.err(
                            format!("Attribute or subscript must be associated to an identifier, another attribute or another subscript."),
                            &err.range(),
                        )
                    }
                }
            }
            ast::Expr::Subscript(s) => {
                match s.value.as_ref() {
                    ast::Expr::Attribute(a) => self.get_lhs_name::<IS_CNST>(&ast::Expr::from(a.clone())),
                    ast::Expr::Subscript(ss) => self.get_lhs_name::<IS_CNST>(&ast::Expr::from(ss.clone())),
                    ast::Expr::Name(n) => Ok(n.clone()),
                    err => {
                        self.err(
                            format!("Attribute or subscript must be associated to an identifier, another attribute or another subscript."),
                            &err.range(),
                        )
                    }
                }
            }
            ast::Expr::Name(n) => Ok(n.clone()),
            err => {
                self.err(
                    format!("Incorrect variable assignment."),
                    &err.range(),
                )
            }
        }
    }

    fn lhs_type_eval_expr<const IS_CNST: bool>(
        &self,
        val: &ast::Expr,
        ty: Ty,
    ) -> Result<Ty, String> {
        match val {
            ast::Expr::Subscript(s) => {
                let ty: Ty = self.lhs_type_eval_expr::<IS_CNST>(&s.value, ty)?;
                match ty {
                    Ty::Array(sz, ity) => match *s.slice {
                        ast::Expr::Slice(_) => Ok(Ty::Array(sz, ity)),
                        _ => Ok(*ity),
                    }
                    ty => Err(format!("Attempted array access on non-Array type {ty}")),
                }
            }
            ast::Expr::Attribute(a) => {
                let ty: Ty = self.lhs_type_eval_expr::<IS_CNST>(&a.value, ty)?;
                match ty {
                    Ty::DataClass(nm, map) => map
                        .search(a.attr.as_str())
                        .map(|r| r.1.clone())
                        .ok_or_else(|| {
                            format!("No such attribute {} of class {nm}", a.attr.as_str())
                        }),
                    ty => Err(format!("Attempted member access on non-Class type {ty}")),
                }
            }
            ast::Expr::Name(_) => Ok(ty),
            err => {
                self.err(
                    format!("Only arrays and object members can be accessed."),
                    &err.range(),
                )
            }
        }
    }

    fn lhs_type<const IS_CNST: bool>(
        &self,
        assign: &ast::Stmt,
    ) -> Result<Ty, String> {
        match assign {
            // Note that right now we only support single assignments.
            // Multi-assign would be nice to add in the future.
            ast::Stmt::Assign(a) => {
                let name = self.get_lhs_name::<IS_CNST>(&a.targets[0])?;
                let t = self.identifier_impl_::<IS_CNST>(&name)?;
                self.lhs_type_eval_expr::<IS_CNST>(&a.targets[0], t.ty)
            }
            ast::Stmt::AugAssign(a) => {
                let name = self.get_lhs_name::<IS_CNST>(&a.target)?;
                let t = self.identifier_impl_::<IS_CNST>(&name)?;
                self.lhs_type_eval_expr::<IS_CNST>(&a.target, t.ty)
            }
            ast::Stmt::AnnAssign(t) => self.type_impl_::<IS_CNST>(&t.annotation),
            err => {
                self.err(
                    format!("Incorrect variable assignment."),
                    &err.range(),
                )
            }
        }
    }

    fn lhs_ty_put(&self, lhs_ty: Option<Ty>) {
        self.lhs_ty.replace(lhs_ty);
    }

    fn lhs_ty_take(&self) -> Option<Ty> {
        self.lhs_ty.borrow_mut().take()
    }

    fn enter_scope_impl_<const IS_CNST: bool>(&self) {
        if IS_CNST {
            self.cvar_enter_scope()
        } else {
            self.circ_enter_scope()
        }
    }

    fn cvar_enter_scope(&self) {
        assert!(!self.cvars_stack.borrow().is_empty());
        self.cvars_stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .push(HashMap::new());
    }

    fn exit_scope_impl_<const IS_CNST: bool>(&self) {
        if IS_CNST {
            self.cvar_exit_scope()
        } else {
            self.circ_exit_scope()
        }
    }

    fn cvar_exit_scope(&self) {
        assert!(!self.cvars_stack.borrow().last().unwrap().is_empty());
        self.cvars_stack.borrow_mut().last_mut().unwrap().pop();
    }

    fn cvar_enter_function(&self) {
        self.cvars_stack.borrow_mut().push(Vec::new());
        self.cvar_enter_scope();
    }

    fn cvar_exit_function(&self) {
        self.cvars_stack.borrow_mut().pop();
    }

    fn cvar_assign(&self, name: &str, val: PyTerm) -> Result<(), String> {
        assert!(!self.cvars_stack.borrow().last().unwrap().is_empty());
        self.cvars_stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .iter_mut()
            .rev()
            .find_map(|v| v.get_mut(name))
            .map(|old_val| {
                *old_val = val;
            })
            .ok_or_else(|| format!("Const assign failed: no variable {name} in scope"))
    }

    fn cvar_declare_init(&self, name: String, ty: &Ty, val: PyTerm) -> Result<(), String> {
        assert!(!self.cvars_stack.borrow().last().unwrap().is_empty());
        if val.type_() != ty {
            return Err(format!(
                "Const decl_init: {} type mismatch: expected {}, got {}",
                name,
                ty,
                val.type_()
            ));
        }
        self.cvars_stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .insert(name, val);
        Ok(())
    }

    fn cvar_declare(&self, name: String, ty: &Ty) -> Result<(), String> {
        self.cvar_declare_init(name, ty, ty.default())
    }

    fn cvar_lookup(&self, name: &str) -> Option<PyTerm> {
        if let Some(st) = self.cvars_stack.borrow().last() {
            st.iter().rev().find_map(|v| v.get(name).cloned())
        } else {
            None
        }
    }

    fn ret_ty_stack_push<const IS_CNST: bool>(
        &self,
        fn_def: &ast::StmtFunctionDef,
    ) -> Result<(), String> {
        // Maybe there is a way to do this more nicely/efficiently,
        // but for now this works.
        // Also, this does not work for multi-returns since type_impl_
        // does not handle Tuples yet.
        let ty = fn_def
            .returns
            .as_ref()
            .map(|idx| *idx.clone())
            .map(|ty| self.type_impl_::<IS_CNST>(&ty))
            .transpose()?
            .unwrap_or(Ty::Bool);
        self.ret_ty_stack.borrow_mut().push(ty);
        Ok(())
    }

    fn ret_ty_stack_pop(&self) {
        self.ret_ty_stack.borrow_mut().pop();
    }

    fn ret_ty_stack_last(&self) -> Option<Ty> {
        self.ret_ty_stack.borrow().last().cloned()
    }

    fn crets_push(&self, ret: PyTerm) {
        self.crets_stack.borrow_mut().push(ret)
    }

    fn crets_pop(&self) -> PyTerm {
        assert!(!self.crets_stack.borrow().is_empty());
        self.crets_stack.borrow_mut().pop().unwrap()
    }

    fn const_decl_(&mut self, c: &mut ast::StmtAnnAssign) {
        // We assume that any annotated assignment in the main body is
        // a constant declaration (since we don't have a const keyword in Python).
        // Moreover, we do very minimal type checking since we haven't implemented
        // the right machinery for doing so. This will be done different in the future.

        // make sure that this wasn't already an important const name
        let ast::Expr::Name(n) = c.target.as_ref() else {
            self.err(
                format!("No name provided for const assignment."),
                &c.target.range(),
            )
        };
        if self
            .cur_import_map()
            .map(|m| m.contains_key(n.id.as_str()))
            .unwrap_or(false)
        {
            self.err(
                format!("Constant {} clashes with import of same name", n.id.as_str()),
                &c.range(),
            );
        }
        // Do we really need Python visitors/mutators?
        // Maybe use them where applicable in a future code refactor.

        // check that constant variable assignment has valid type
        let ctype = self.unwrap(self.type_impl_::<true>(&c.annotation), &c.annotation.range());

        // evaluate the expression and check the resulting type
        let value = self
            .expr_impl_::<true>(c.value.clone().unwrap().as_ref())
            .unwrap_or_else(|e| self.err(e, &c.value.clone().unwrap().range()));

        if &ctype != value.type_() {
            self.err(
                format!(
                    "Type mismatch in constant definition: expected {:?}, got {:?}",
                    ctype,
                    value.type_()
                ),
                &c.range(),
            );
        }
        // insert into constant map
        if self
            .constants
            .get_mut(self.file_stack.borrow().last().unwrap())
            .unwrap()
            .insert(n.id.to_string(), (*c.annotation.clone(), value))
            .is_some()
        {
            self.err(format!("Constant {} redefined", &n.id.as_str()), &c.range());
        }
    }

    fn type_(&self, t: &ast::Expr) -> Ty {
        self.unwrap(self.type_impl_::<false>(&t.clone()), &t.range())
    }

    fn type_impl_<const IS_CNST: bool>(&self, t: &ast::Expr) -> Result<Ty, String> {
        if IS_CNST {
            debug!("Const type: {:?}", t);
        } else {
            debug!("Type: {:?}", t);
        }

        // This should seriously be refactored at some point.
        match t {
            ast::Expr::Subscript(s) => {
                let ast::Expr::Name(n) = s.value.as_ref() else {
                    self.err(
                        format!("Error while interpreting type annotation of assignment. Subscript identifier could not be found."),
                        &s.value.range(),
                    )
                };
                if n.id.as_str() == "Array" {
                    let ast::Expr::Tuple(t) = s.slice.as_ref() else {
                        self.err(
                            format!("Array type has 2 fields. The first position should be the datatype, and the second position should be the size."),
                            &s.slice.range(),
                        )
                    };
                    let expr = &t.elts[0];
                    let dim = &t.elts[1];
                    if t.elts.len() != 2 {
                        self.err(
                            format!("Array type has 2 fields. The first position should be the datatype, and the second position should be the size."),
                            &t.range(),
                        )
                    }
                    if let ast::Expr::Constant(c) = dim {
                        let d = c.value
                            .as_int()
                            .unwrap()
                            .to_u32_digits().1
                            .into_iter()
                            .next()
                            .unwrap_or(0);
                        let b = self.type_impl_::<IS_CNST>(expr);
                        Ok(Ty::Array(d.try_into().unwrap(), Box::new(b?)))
                    } else {
                        // I suppose the static bound could be inferred, so need not 
                        // necessarily be a constant. Improve this in the future.
                        self.err(
                            format!("The second field of the array type must be a constant."),
                            &dim.range(),
                        )
                    }
                } else if n.id.as_str() == "Private" || n.id.as_str() == "Public" {
                    // Accessibilities don't have a type associated to it, so just continue
                    return self.type_impl_::<IS_CNST>(&ast::Expr::from(*s.slice.clone()));
                } else {
                    // If we have subscript for any other identifier, then that is wrong.
                    // Atm we only want this for arrays (maybe other types in the future).
                    self.err(
                        format!("Invalid type."),
                        &s.slice.range(),
                    )
                }
            }
            ast::Expr::Name(n) => {
                let name = n.id.as_str();
                let (class_path, class_name) = self.deref_import(name);
                if name == "float" {
                    self.err(
                        format!("Floats are not supported yet."),
                        &n.range(),
                    )
                } else if name == "complex" {
                    self.err(
                        format!("Complex numbers are not supported yet."),
                        &n.range(),
                    )
                } else if name == "bool" {
                    Ok(Ty::Bool)
                } else if name == "field" {
                    Ok(Ty::Field)
                } else if name == "int" {
                    Ok(Ty::Uint(32))
                } else if let Some(m) = self.classes_and_tys.get(&class_path) {
                    // If class_name is a defined class, 
                    let (def, path) = self.get_class_or_type(&class_name).ok_or_else(|| {
                        format!(
                            "No such class {} (did you bring it into scope?)",
                            &class_name
                        )
                    })?;
                    if m.contains_key(&class_name) {
                        self.file_stack_push(path);
                        let ty = match def {
                            Ok(sdef) => Ty::new_class(
                                sdef.name.to_string(),
                                sdef.body
                                    .iter()
                                    .map::<Result<_, String>, _>(|f| {
                                        if let ast::Stmt::AnnAssign(a) = f {
                                            if let ast::Expr::Name(n) = a.target.as_ref() {
                                                Ok((n.id.to_string(), self.type_impl_::<IS_CNST>(&a.annotation)?))
                                            } else {
                                                self.err(
                                                    format!("Missing name of struct field."),
                                                    &a.target.range(),
                                                )
                                            }
                                        } else {
                                            self.err(
                                                format!("Only annotated assignments are allowed in class definitions."),
                                                &f.range(),
                                            )
                                        }
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                            ),
                            Err(tdef) => self.type_impl_::<IS_CNST>(&tdef.value)?,
                        };
                        self.file_stack_pop();
                        Ok(ty)
                    } else {
                        self.err(
                            format!("Invalid type."),
                            &n.range(),
                        )
                    }
                    // I'm too lazy to avoid code repetition atm.
                } else {
                    self.err(
                        format!("Invalid type."),
                        &n.range(),
                    )
                }
            }
            err => {
                self.err(
                    format!("Invalid type."),
                    &err.range(),
                )
            }
        }
    }

    fn visit_files(&mut self, entry_point: &String) {
        // 1. go through includes and return a toposorted visit order for remaining processing
        let files = self.visit_imports(entry_point);

        // 2. visit constant, class, and function defs ; infer types
        self.visit_body(files);
    }

    fn visit_imports(&mut self, entry_point: &String) -> Vec<PathBuf> {
        use petgraph::algo::toposort;
        use petgraph::graph::{DefaultIx, DiGraph, NodeIndex};
        let asts = std::mem::take(&mut self.asts);

        // we use the graph to toposort the includes and the map to go from PathBuf to NodeIdx
        let mut ig = DiGraph::<PathBuf, ()>::with_capacity(asts.len(), asts.len());
        let mut gn = HashMap::<PathBuf, NodeIndex<DefaultIx>>::with_capacity(asts.len());

        for (p, f) in asts.iter() {
            self.file_stack_push(p.to_owned());
            let mut imap = HashMap::new();

            if !gn.contains_key(p) {
                gn.insert(p.to_owned(), ig.add_node(p.to_owned()));
            }

            let ast::Mod::Module(f_mod) = f else {
                panic!("Loaded file does not implement a module.");
            };

            for d in f_mod.body.iter() {
                // XXX(opt) retain() declarations instead? if we don't need them, saves allocs
                let (src_path, src_names, dst_names, i_span) = match d {
                    // Multi-import is not supported yet.
                    ast::Stmt::Import(m) => (
                        m.names[0].name.to_string(),
                        vec![entry_point.to_string()],
                        vec![m
                            .names[0]
                            .asname
                            .as_ref()
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| {
                                PathBuf::from(m.names[0].name.to_string())
                                    .file_stem()
                                    .unwrap_or_else(|| panic!("Bad import: {}", m.names[0].name.as_str()))
                                    .to_string_lossy()
                                    .to_string()
                            })],
                        m.range(),
                    ),
                    ast::Stmt::ImportFrom(m) => (
                        m.module.as_ref().unwrap().to_string(),
                        m.names.iter().map(|s| s.name.to_string()).collect(),
                        m.names
                            .iter()
                            .map(|s| {
                                s.asname
                                    .as_ref()
                                    .map(|a| a.to_string())
                                    .unwrap_or_else(|| s.name.to_string())
                            })
                            .collect(),
                        m.range(),
                    ),
                    _ => continue,
                };
                assert!(!src_names.is_empty());
                let child = src_path.replace(".", "/");
                let abs_src_path = self.stdlib.canonicalize(&self.cur_dir(), child.as_str());
                debug!(
                    "Import of {:?} from {} as {:?}",
                    src_names,
                    abs_src_path.display(),
                    dst_names
                );
                src_names
                    .into_iter()
                    .zip(dst_names.into_iter())
                    .for_each(|(sn, dn)| {
                        if imap.contains_key(&dn) {
                            self.err(format!("Import {dn} redeclared"), &i_span);
                        }
                        assert!(imap.insert(dn, (abs_src_path.clone(), sn)).is_none());
                    });

                // add included -> includer edge for later toposort
                if !gn.contains_key(&abs_src_path) {
                    gn.insert(abs_src_path.clone(), ig.add_node(abs_src_path.clone()));
                }
                ig.add_edge(*gn.get(&abs_src_path).unwrap(), *gn.get(p).unwrap(), ());
            }

            let p = self.file_stack_pop().unwrap();
            self.import_map.insert(p, imap);
        }
        self.asts = asts;

        // flatten the import map, i.e., a -> b -> c becomes a -> c
        self.flatten_import_map();

        toposort(&ig, None)
            .unwrap_or_else(|e| {
                use petgraph::dot::{Config, Dot};
                panic!(
                    "Import graph is cyclic!: {:?}\n{:?}\n",
                    e,
                    Dot::with_config(&ig, &[Config::EdgeNoLabel])
                )
            })
            .iter()
            .map(|idx| std::mem::take(ig.node_weight_mut(*idx).unwrap()))
            .filter(|p| self.asts.contains_key(p))
            .collect()
    }

    fn flatten_import_map(&mut self) {
        // create a new map
        let mut new_map = HashMap::with_capacity(self.import_map.len());
        self.import_map.keys().for_each(|k| {
            new_map.insert(k.clone(), HashMap::new());
        });

        let mut visited = Vec::new();
        for (fname, map) in &self.import_map {
            for (iname, (nv, iv)) in map.iter() {
                // unwrap is safe because of new_map's initialization above
                if new_map.get(fname).unwrap().contains_key(iname) {
                    // visited this value already as part of a prior pointer chase
                    continue;
                }

                // chase the pointer, writing down every visited key along the way
                visited.clear();
                visited.push((fname, iname));
                let mut n = nv;
                let mut i = iv;
                while let Some((nn, ii)) = self.import_map.get(n).and_then(|m| m.get(i)) {
                    visited.push((n, i));
                    n = nn;
                    i = ii;
                }

                // map every visited key to the final value in the ptr chase
                visited.iter().for_each(|&(nn, ii)| {
                    new_map
                        .get_mut(nn)
                        .unwrap()
                        .insert(ii.clone(), (n.clone(), i.clone()));
                });
            }
        }

        self.import_map = new_map;
    }

    fn visit_body(&mut self, files: Vec<PathBuf>) {
        let mut t = std::mem::take(&mut self.asts);
        for p in files {
            self.constants.insert(p.clone(), HashMap::new());
            self.classes_and_tys.insert(p.clone(), HashMap::new());
            self.functions.insert(p.clone(), HashMap::new());
            self.file_stack_push(p.clone());
            let ast::Mod::Module(m) = t.get_mut(&p).unwrap() else {
                panic!("Loaded file {} does not implement module.", p.display())
            };
            for d in m.body.iter_mut() {
                match d {
                    ast::Stmt::FunctionDef(f) => {
                        debug!("processing decl: fn {} in {}", f.name.as_str(), p.display());
                        let f_ast = f.clone();

                        // Do not check return type of embedded functions
                        let (f_path, _) = self.deref_import(f.name.as_str());
                        if !self.stdlib.is_embed(&f_path) {
                            if self.function_ret_type(&f_ast).len() != 1 {
                                // functions MUST return exactly 1 value
                                self.err(
                                    format!(
                                        "Functions must return exactly 1 value; {} returns {}",
                                        &f_ast.name.as_str(),
                                        self.function_ret_type(&f_ast).len(),
                                    ),
                                    &f.range(),
                                );
                            }
                        }

                        if self
                            .functions
                            .get_mut(self.file_stack.borrow().last().unwrap())
                            .unwrap()
                            .insert(f.name.to_string(), f_ast)
                            .is_some()
                        {
                            self.err(format!("Function {} redefined", &f.name.as_str()), &f.range());
                        }
                    }
                    ast::Stmt::AsyncFunctionDef(a) => {
                        self.err(
                            format!("Async function def statement is not supported yet."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::ClassDef(c) => {
                        debug!("processing decl: class {} in {}", c.name.as_str(), p.display());
                        let c_ast = c.clone();

                        if self
                            .classes_and_tys
                            .get_mut(self.file_stack.borrow().last().unwrap())
                            .unwrap()
                            .insert(c.name.to_string(), Ok(c_ast))
                            .is_some()
                        {
                            self.err(
                                format!("Struct {} defined over existing name", c.name.as_str()),
                                &c.range(),
                            );
                        }
                    }
                    ast::Stmt::Return(r) => {
                        self.err(
                            format!("Return statement must live inside of a function."),
                            &r.range(),
                        )
                    }
                    ast::Stmt::Delete(dd) => {
                        self.err(
                            format!("Delete statement is not supported yet."),
                            &dd.range(),
                        )
                    }
                    ast::Stmt::Assign(a) => {
                        self.err(
                            format!("Constant declaration needs type annotation."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::TypeAlias(t) => {
                        self.err(
                            format!("Type aliases are not supported yet."),
                            &t.range(),
                        )
                    }
                    ast::Stmt::AugAssign(a) => {
                        self.err(
                            format!("Augmented assignment statement must live inside of a function."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::AnnAssign(a) => {
                        let ast::Expr::Name(n) = a.target.as_ref() else {
                            self.err(
                                format!("No name provided for const assignment."),
                                &a.target.range(),
                            )
                        };
                        debug!("processing decl: const {} in {}", n.id.as_str(), p.display());
                        self.const_decl_(a);
                    }
                    ast::Stmt::For(f) => {
                        self.err(
                            format!("For statement must live inside of a function."),
                            &f.range(),
                        )
                    }
                    ast::Stmt::AsyncFor(a) => {
                        self.err(
                            format!("Async for statement must live inside of a function."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::While(w) => {
                        self.err(
                            format!("While statement must live inside of a function."),
                            &w.range(),
                        )
                    }
                    ast::Stmt::If(i) => {
                        self.err(
                            format!("If statement must live inside of a function."),
                            &i.range(),
                        )
                    }
                    ast::Stmt::With(w) => {
                        self.err(
                            format!("With statement must live inside of a function."),
                            &w.range(),
                        )
                    }
                    ast::Stmt::AsyncWith(a) => {
                        self.err(
                            format!("Async with statement must live inside of a function."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::Match(m) => {
                        self.err(
                            format!("Match statement must live inside of a function."),
                            &m.range(),
                        )
                    }
                    ast::Stmt::Raise(r) => {
                        self.err(
                            format!("Raise statement must live inside of a function."),
                            &r.range(),
                        )
                    }
                    ast::Stmt::Try(t) => {
                        self.err(
                            format!("Try statement must live inside of a function."),
                            &t.range(),
                        )
                    }
                    ast::Stmt::TryStar(t) => {
                        self.err(
                            format!("Try star statement must live inside of a function."),
                            &t.range(),
                        )
                    }
                    ast::Stmt::Assert(a) => {
                        self.err(
                            format!("Assert statement must live inside of a function."),
                            &a.range(),
                        )
                    }
                    ast::Stmt::Import(_) => (), // already handled in visit_imports.
                    ast::Stmt::ImportFrom(_) => (), // already handled in visit_imports.
                    ast::Stmt::Global(g) => {
                        self.err(
                            format!("Global statement must live inside of a function."),
                            &g.range(),
                        )
                    }
                    ast::Stmt::Nonlocal(n) => {
                        self.err(
                            format!("Nonlocal statement must live inside of a function."),
                            &n.range(),
                        )
                    }
                    ast::Stmt::Expr(e) => {
                        self.err(
                            format!("Expression statements are not supported."),
                            &e.range(),
                        )
                    }
                    ast::Stmt::Pass(pp) => {
                        self.err(
                            format!("Pass statement must live inside of a function."),
                            &pp.range(),
                        )
                    }
                    ast::Stmt::Break(b) => {
                        self.err(
                            format!("Break statement must live inside of a function."),
                            &b.range(),
                        )
                    }
                    ast::Stmt::Continue(c) => {
                        self.err(
                            format!("Continue statement must live inside of a function."),
                            &c.range(),
                        )
                    }
                }
            }
            self.file_stack_pop();
        }
        self.asts = t;
    }

    fn get_function(&self, fn_id: &str) -> Option<&ast::StmtFunctionDef> {
        let (f_path, f_name) = self.deref_import(fn_id);
        self.functions.get(&f_path).and_then(|m| m.get(&f_name))
    }

    fn get_class_or_type(
        &self,
        class_id: &str,
    ) -> Option<(
        Result<&ast::StmtClassDef, &ast::StmtTypeAlias>,
        PathBuf,
    )> {
        let (s_path, s_name) = self.deref_import(class_id);
        self.classes_and_tys
            .get(&s_path)
            .and_then(|m| m.get(&s_name))
            .map(|m| (m.as_ref(), s_path))
    }

    fn assert(&self, asrt: Term) {
        debug_assert!(matches!(check(&asrt), Sort::Bool));
        if self.isolate_asserts {
            let path = self.circ_condition();
            self.assertions
                .borrow_mut()
                .push(term![IMPLIES; path, asrt]);
        } else {
            self.assertions.borrow_mut().push(asrt);
        }
    }

    /*** circify wrapper functions (hides RefCell) ***/

    fn circ_enter_condition(&self, cond: Term) {
        if self.isolate_asserts {
            self.circ.borrow_mut().enter_condition(cond).unwrap();
        }
    }

    fn circ_exit_condition(&self) {
        if self.isolate_asserts {
            self.circ.borrow_mut().exit_condition()
        }
    }

    fn circ_condition(&self) -> Term {
        self.circ.borrow().condition()
    }

    fn circ_return_(&self, ret: Option<PyTerm>) -> Result<(), CircError> {
        self.circ.borrow_mut().return_(ret)
    }

    fn circ_enter_fn(&self, f_name: String, ret_ty: Option<Ty>) {
        self.circ.borrow_mut().enter_fn(f_name, ret_ty)
    }

    fn circ_exit_fn(&self) -> Option<Val<PyTerm>> {
        self.circ.borrow_mut().exit_fn()
    }

    fn circ_enter_scope(&self) {
        self.circ.borrow_mut().enter_scope()
    }

    fn circ_exit_scope(&self) {
        self.circ.borrow_mut().exit_scope()
    }

    fn circ_declare_input(
        &self,
        name: String,
        ty: &Ty,
        vis: PyVis,
        precomputed_value: Option<PyTerm>,
        mangle_name: bool,
    ) -> Result<PyTerm, CircError> {
        match vis {
            PyVis::Public => {
                self.circ
                    .borrow_mut()
                    .declare_input(name, ty, None, precomputed_value, mangle_name)
            }
            PyVis::Private(i) => self.circ.borrow_mut().declare_input(
                name,
                ty,
                Some(i),
                precomputed_value,
                mangle_name,
            ),
            PyVis::Committed => {
                let size = match ty {
                    Ty::Array(size, _) => *size,
                    _ => panic!(),
                };
                Ok(self.circ.borrow_mut().start_persistent_array(
                    &name,
                    size,
                    default_field(),
                    crate::front::proof::PROVER_ID,
                ))
            }
        }
    }

    fn circ_declare_init(&self, name: String, ty: Ty, val: Val<PyTerm>) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow_mut().declare_init(name, ty, val)
    }

    fn circ_get_value(&self, loc: Loc) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow().get_value(loc)
    }

    fn circ_assign(&self, loc: Loc, val: Val<PyTerm>) -> Result<Val<PyTerm>, CircError> {
        self.circ.borrow_mut().assign(loc, val)
    }
}

fn range_to_string(s: &TextRange, path: PathBuf) -> String {
    if let Ok(file_contents) = fs::read_to_string(&path) {
        let mut corrected_contents = file_contents.clone();
        filter_out_zk_ignore(&mut corrected_contents);
        if s.start().to_usize() <= corrected_contents.len() && s.end().to_usize() <= corrected_contents.len() {
            corrected_contents[s.start().to_usize()..s.end().to_usize()].to_string()
        } else {
            panic!("TextRange is out of bounds.")
        }
    } else {
        panic!("Failed to read the file contents in {}", &path.canonicalize().unwrap().display())
    }
}

fn range_before_filter(s: &TextRange, path: PathBuf) -> TextRange {
    if let Ok(file_contents) = fs::read_to_string(&path) {
        let mut corrected_contents = file_contents.clone();
        let filtered_ranges = filter_out_zk_ignore(&mut corrected_contents);
        
        // Sum up all TextRange values in filtered_ranges up to s
        let offset: TextSize = filtered_ranges
            .iter()
            .take_while(|&range| range.end() <= s.start())
            .map(|range| range.len() + TextSize::from(1))
            .sum();

        // Compute the updated s
        s + offset
    } else {
        panic!("Failed to read the file contents in {}", &path.canonicalize().unwrap().display())
    }
}

fn line_from_range(range: TextRange, path: PathBuf) -> usize {
    if let Ok(file_contents) = fs::read_to_string(&path) {
        let start_offset = range.start();
        let text_before_range = &file_contents[0..start_offset.into()];
        text_before_range.chars().filter(|&c| c == '\n').count() + 1
    } else {
        panic!("Failed to read the file contents in {}", &path.canonicalize().unwrap().display())
    }
}