//! AST Walker for rustpython_parser_ast

use super::{PyVisitorMut, PyVisitorResult};
use ast::text_size::TextRange;
use rustpython_parser::ast as ast;

pub fn walk_stmt<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::Stmt<TextRange>,
) -> PyVisitorResult {
    match node {
        ast::Stmt::FunctionDef(data) => visitor.visit_stmt_function_def(data),
        ast::Stmt::AsyncFunctionDef(data) => visitor.visit_stmt_async_function_def(data),
        ast::Stmt::ClassDef(data) => visitor.visit_stmt_class_def(data),
        ast::Stmt::Return(data) => visitor.visit_stmt_return(data),
        ast::Stmt::Delete(data) => visitor.visit_stmt_delete(data),
        ast::Stmt::Assign(data) => visitor.visit_stmt_assign(data),
        ast::Stmt::TypeAlias(data) => visitor.visit_stmt_type_alias(data),
        ast::Stmt::AugAssign(data) => visitor.visit_stmt_aug_assign(data),
        ast::Stmt::AnnAssign(data) => visitor.visit_stmt_ann_assign(data),
        ast::Stmt::For(data) => visitor.visit_stmt_for(data),
        ast::Stmt::AsyncFor(data) => visitor.visit_stmt_async_for(data),
        ast::Stmt::While(data) => visitor.visit_stmt_while(data),
        ast::Stmt::If(data) => visitor.visit_stmt_if(data),
        ast::Stmt::With(data) => visitor.visit_stmt_with(data),
        ast::Stmt::AsyncWith(data) => visitor.visit_stmt_async_with(data),
        ast::Stmt::Match(data) => visitor.visit_stmt_match(data),
        ast::Stmt::Raise(data) => visitor.visit_stmt_raise(data),
        ast::Stmt::Try(data) => visitor.visit_stmt_try(data),
        ast::Stmt::TryStar(data) => visitor.visit_stmt_try_star(data),
        ast::Stmt::Assert(data) => visitor.visit_stmt_assert(data),
        ast::Stmt::Import(data) => visitor.visit_stmt_import(data),
        ast::Stmt::ImportFrom(data) => visitor.visit_stmt_import_from(data),
        ast::Stmt::Global(data) => visitor.visit_stmt_global(data),
        ast::Stmt::Nonlocal(data) => visitor.visit_stmt_nonlocal(data),
        ast::Stmt::Expr(data) => visitor.visit_stmt_expr(data),
        ast::Stmt::Pass(data) => visitor.visit_stmt_pass(data),
        ast::Stmt::Break(data) => visitor.visit_stmt_break(data),
        ast::Stmt::Continue(data) => visitor.visit_stmt_continue(data),
    }
}

pub fn walk_stmt_function_def<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtFunctionDef<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.args;
        visitor.visit_arguments(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.decorator_list {
        visitor.visit_expr(value)?;
    }
    if let Some(value) = node.returns {
        visitor.visit_expr(*value)?;
    }
    for value in node.type_params {
        visitor.visit_type_param(value)?;
    }
    Ok(())
}

pub fn walk_stmt_async_function_def<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAsyncFunctionDef<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.args;
        visitor.visit_arguments(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.decorator_list {
        visitor.visit_expr(value)?;
    }
    if let Some(value) = node.returns {
        visitor.visit_expr(*value)?;
    }
    for value in node.type_params {
        visitor.visit_type_param(value)?;
    }
    Ok(())
}

pub fn walk_stmt_class_def<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtClassDef<TextRange>,
) -> PyVisitorResult {
    for value in node.bases {
        visitor.visit_expr(value)?;
    }
    for value in node.keywords {
        visitor.visit_keyword(value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.decorator_list {
        visitor.visit_expr(value)?;
    }
    for value in node.type_params {
        visitor.visit_type_param(value)?;
    }
    Ok(())
}

pub fn walk_stmt_return<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtReturn<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.value {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_delete<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtDelete<TextRange>,
) -> PyVisitorResult {
    for value in node.targets {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_stmt_assign<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAssign<TextRange>,
) -> PyVisitorResult {
    for value in node.targets {
        visitor.visit_expr(value)?;
    }
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_type_alias<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtTypeAlias<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.name;
        visitor.visit_expr(*value)?;
    }
    for value in node.type_params {
        visitor.visit_type_param(value)?;
    }
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_aug_assign<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAugAssign<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.target;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_ann_assign<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAnnAssign<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.target;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.annotation;
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.value {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_for<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtFor<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.target;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.iter;
        visitor.visit_expr(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_async_for<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAsyncFor<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.target;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.iter;
        visitor.visit_expr(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_while<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtWhile<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.test;
        visitor.visit_expr(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_if<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtIf<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.test;
        visitor.visit_expr(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_with<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtWith<TextRange>,
) -> PyVisitorResult {
    for value in node.items {
        visitor.visit_withitem(value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_async_with<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAsyncWith<TextRange>,
) -> PyVisitorResult {
    for value in node.items {
        visitor.visit_withitem(value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_match<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtMatch<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.subject;
        visitor.visit_expr(*value)?;
    }
    for value in node.cases {
        visitor.visit_match_case(value)?;
    }
    Ok(())
}

pub fn walk_stmt_raise<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtRaise<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.exc {
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.cause {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_try<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtTry<TextRange>,
) -> PyVisitorResult {
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.handlers {
        visitor.visit_excepthandler(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    for value in node.finalbody {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_try_star<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtTryStar<TextRange>,
) -> PyVisitorResult {
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    for value in node.handlers {
        visitor.visit_excepthandler(value)?;
    }
    for value in node.orelse {
        visitor.visit_stmt(value)?;
    }
    for value in node.finalbody {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_stmt_assert<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtAssert<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.test;
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.msg {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_stmt_import<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtImport<TextRange>,
) -> PyVisitorResult {
    for value in node.names {
        visitor.visit_alias(value)?;
    }
    Ok(())
}

pub fn walk_stmt_import_from<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtImportFrom<TextRange>,
) -> PyVisitorResult {
    for value in node.names {
        visitor.visit_alias(value)?;
    }
    Ok(())
}

pub fn walk_stmt_expr<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::StmtExpr<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::Expr<TextRange>,
) -> PyVisitorResult {
    match node {
        ast::Expr::BoolOp(data) => visitor.visit_expr_bool_op(data),
        ast::Expr::NamedExpr(data) => visitor.visit_expr_named_expr(data),
        ast::Expr::BinOp(data) => visitor.visit_expr_bin_op(data),
        ast::Expr::UnaryOp(data) => visitor.visit_expr_unary_op(data),
        ast::Expr::Lambda(data) => visitor.visit_expr_lambda(data),
        ast::Expr::IfExp(data) => visitor.visit_expr_if_exp(data),
        ast::Expr::Dict(data) => visitor.visit_expr_dict(data),
        ast::Expr::Set(data) => visitor.visit_expr_set(data),
        ast::Expr::ListComp(data) => visitor.visit_expr_list_comp(data),
        ast::Expr::SetComp(data) => visitor.visit_expr_set_comp(data),
        ast::Expr::DictComp(data) => visitor.visit_expr_dict_comp(data),
        ast::Expr::GeneratorExp(data) => visitor.visit_expr_generator_exp(data),
        ast::Expr::Await(data) => visitor.visit_expr_await(data),
        ast::Expr::Yield(data) => visitor.visit_expr_yield(data),
        ast::Expr::YieldFrom(data) => visitor.visit_expr_yield_from(data),
        ast::Expr::Compare(data) => visitor.visit_expr_compare(data),
        ast::Expr::Call(data) => visitor.visit_expr_call(data),
        ast::Expr::FormattedValue(data) => visitor.visit_expr_formatted_value(data),
        ast::Expr::JoinedStr(data) => visitor.visit_expr_joined_str(data),
        ast::Expr::Constant(data) => visitor.visit_expr_constant(data),
        ast::Expr::Attribute(data) => visitor.visit_expr_attribute(data),
        ast::Expr::Subscript(data) => visitor.visit_expr_subscript(data),
        ast::Expr::Starred(data) => visitor.visit_expr_starred(data),
        ast::Expr::Name(data) => visitor.visit_expr_name(data),
        ast::Expr::List(data) => visitor.visit_expr_list(data),
        ast::Expr::Tuple(data) => visitor.visit_expr_tuple(data),
        ast::Expr::Slice(data) => visitor.visit_expr_slice(data),
    }
}

pub fn walk_expr_bool_op<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprBoolOp<TextRange>,
) -> PyVisitorResult {
    for value in node.values {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_named_expr<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprNamedExpr<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.target;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_bin_op<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprBinOp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.left;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.right;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_unary_op<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprUnaryOp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.operand;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_lambda<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprLambda<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.args;
        visitor.visit_arguments(*value)?;
    }
    {
        let value = node.body;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_if_exp<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprIfExp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.test;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.body;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.orelse;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_dict<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprDict<TextRange>,
) -> PyVisitorResult {
    for value in node.keys.into_iter().flatten() {
        visitor.visit_expr(value)?;
    }
    for value in node.values {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_set<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprSet<TextRange>,
) -> PyVisitorResult {
    for value in node.elts {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_list_comp<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprListComp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.elt;
        visitor.visit_expr(*value)?;
    }
    for value in node.generators {
        visitor.visit_comprehension(value)?;
    }
    Ok(())
}

pub fn walk_expr_set_comp<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprSetComp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.elt;
        visitor.visit_expr(*value)?;
    }
    for value in node.generators {
        visitor.visit_comprehension(value)?;
    }
    Ok(())
}

pub fn walk_expr_dict_comp<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprDictComp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.key;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    for value in node.generators {
        visitor.visit_comprehension(value)?;
    }
    Ok(())
}

pub fn walk_expr_generator_exp<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprGeneratorExp<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.elt;
        let _ = visitor.visit_expr(*value);
    }
    for value in node.generators {
        visitor.visit_comprehension(value)?;
    }
    Ok(())
}

pub fn walk_expr_await<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprAwait<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_yield<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprYield<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.value {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_yield_from<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprYieldFrom<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_compare<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprCompare<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.left;
        visitor.visit_expr(*value)?;
    }
    for value in node.comparators {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_call<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprCall<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.func;
        visitor.visit_expr(*value)?;
    }
    for value in node.args {
        visitor.visit_expr(value)?;
    }
    for value in node.keywords {
        visitor.visit_keyword(value)?;
    }
    Ok(())
}

pub fn walk_expr_formatted_value<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprFormattedValue<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.format_spec {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_joined_str<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprJoinedStr<TextRange>,
) -> PyVisitorResult {
    for value in node.values {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_attribute<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprAttribute<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_subscript<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprSubscript<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    {
        let value = node.slice;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_starred<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprStarred<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_expr_list<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprList<TextRange>,
) -> PyVisitorResult {
    for value in node.elts {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_tuple<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprTuple<TextRange>,
) -> PyVisitorResult {
    for value in node.elts {
        visitor.visit_expr(value)?;
    }
    Ok(())
}

pub fn walk_expr_slice<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExprSlice<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.lower {
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.upper {
        visitor.visit_expr(*value)?;
    }
    if let Some(value) = node.step {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_excepthandler<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExceptHandler<TextRange>,
) -> PyVisitorResult {
    match node {
        ast::ExceptHandler::ExceptHandler(data) => visitor.visit_excepthandler_except_handler(data),
    }
}

pub fn walk_excepthandler_except_handler<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::ExceptHandlerExceptHandler<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.type_ {
        visitor.visit_expr(*value)?;
    }
    for value in node.body {
        visitor.visit_stmt(value)?;
    }
    Ok(())
}

pub fn walk_pattern<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::Pattern<TextRange>,
) -> PyVisitorResult {
    match node {
        ast::Pattern::MatchValue(data) => visitor.visit_pattern_match_value(data),
        ast::Pattern::MatchSingleton(data) => visitor.visit_pattern_match_singleton(data),
        ast::Pattern::MatchSequence(data) => visitor.visit_pattern_match_sequence(data),
        ast::Pattern::MatchMapping(data) => visitor.visit_pattern_match_mapping(data),
        ast::Pattern::MatchClass(data) => visitor.visit_pattern_match_class(data),
        ast::Pattern::MatchStar(data) => visitor.visit_pattern_match_star(data),
        ast::Pattern::MatchAs(data) => visitor.visit_pattern_match_as(data),
        ast::Pattern::MatchOr(data) => visitor.visit_pattern_match_or(data),
    }
}

pub fn walk_pattern_match_value<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchValue<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.value;
        visitor.visit_expr(*value)?;
    }
    Ok(())
}

pub fn walk_pattern_match_sequence<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchSequence<TextRange>,
) -> PyVisitorResult {
    for value in node.patterns {
        visitor.visit_pattern(value)?;
    }
    Ok(())
}

pub fn walk_pattern_match_mapping<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchMapping<TextRange>,
) -> PyVisitorResult {
    for value in node.keys {
        visitor.visit_expr(value)?;
    }
    for value in node.patterns {
        visitor.visit_pattern(value)?;
    }
    Ok(())
}

pub fn walk_pattern_match_class<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchClass<TextRange>,
) -> PyVisitorResult {
    {
        let value = node.cls;
        visitor.visit_expr(*value)?;
    }
    for value in node.patterns {
        visitor.visit_pattern(value)?;
    }
    for value in node.kwd_patterns {
        visitor.visit_pattern(value)?;
    }
    Ok(())
}

pub fn walk_pattern_match_as<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchAs<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.pattern {
        visitor.visit_pattern(*value)?;
    }
    Ok(())
}

pub fn walk_pattern_match_or<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::PatternMatchOr<TextRange>,
) -> PyVisitorResult {
    for value in node.patterns {
        visitor.visit_pattern(value)?;
    }
    Ok(())
}

pub fn walk_type_param<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::TypeParam<TextRange>,
) -> PyVisitorResult {
    match node {
        ast::TypeParam::TypeVar(data) => visitor.visit_type_param_type_var(data),
        ast::TypeParam::ParamSpec(data) => visitor.visit_type_param_param_spec(data),
        ast::TypeParam::TypeVarTuple(data) => visitor.visit_type_param_type_var_tuple(data),
    }
}

pub fn walk_type_param_type_var<Py: PyVisitorMut>(
    visitor: &mut Py,
    node: ast::TypeParamTypeVar<TextRange>,
) -> PyVisitorResult {
    if let Some(value) = node.bound {
        visitor.visit_expr(*value)?;
    }
    Ok(())
}
