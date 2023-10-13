//! AST Walker adaptation of rustpython_ast implementation

use super::{PyVisitorResult, walkfns::*};
use ast::text_size::TextRange;
use rustpython_parser::ast as ast;

pub trait PyVisitorMut<>: Sized {
    fn visit_stmt(&mut self, node: ast::Stmt<TextRange>) -> PyVisitorResult {
        walk_stmt(self, node)
    }

    fn visit_stmt_function_def(
        &mut self,
        node: ast::StmtFunctionDef<TextRange>,
    ) -> PyVisitorResult {
        walk_stmt_function_def(self, node)
    }

    fn visit_stmt_async_function_def(
        &mut self,
        node: ast::StmtAsyncFunctionDef<TextRange>,
    ) -> PyVisitorResult {
        walk_stmt_async_function_def(self, node)
    }

    fn visit_stmt_class_def(&mut self, node: ast::StmtClassDef<TextRange>) -> PyVisitorResult {
        walk_stmt_class_def(self, node)
    }

    fn visit_stmt_return(&mut self, node: ast::StmtReturn<TextRange>) -> PyVisitorResult {
        walk_stmt_return(self, node)
    }

    fn visit_stmt_delete(&mut self, node: ast::StmtDelete<TextRange>) -> PyVisitorResult {
        walk_stmt_delete(self, node)
    }

    fn visit_stmt_assign(&mut self, node: ast::StmtAssign<TextRange>) -> PyVisitorResult {
        walk_stmt_assign(self, node)
    }

    fn visit_stmt_type_alias(&mut self, node: ast::StmtTypeAlias<TextRange>) -> PyVisitorResult {
        walk_stmt_type_alias(self, node)
    }

    fn visit_stmt_aug_assign(&mut self, node: ast::StmtAugAssign<TextRange>) -> PyVisitorResult {
        walk_stmt_aug_assign(self, node)
    }

    fn visit_stmt_ann_assign(&mut self, node: ast::StmtAnnAssign<TextRange>) -> PyVisitorResult {
        walk_stmt_ann_assign(self, node)
    }

    fn visit_stmt_for(&mut self, node: ast::StmtFor<TextRange>) -> PyVisitorResult {
        walk_stmt_for(self, node)
    }

    fn visit_stmt_async_for(&mut self, node: ast::StmtAsyncFor<TextRange>) -> PyVisitorResult {
        walk_stmt_async_for(self, node)
    }

    fn visit_stmt_while(&mut self, node: ast::StmtWhile<TextRange>) -> PyVisitorResult {
        walk_stmt_while(self, node)
    }

    fn visit_stmt_if(&mut self, node: ast::StmtIf<TextRange>) -> PyVisitorResult {
        walk_stmt_if(self, node)
    }

    fn visit_stmt_with(&mut self, node: ast::StmtWith<TextRange>) -> PyVisitorResult {
        walk_stmt_with(self, node)
    }

    fn visit_stmt_async_with(&mut self, node: ast::StmtAsyncWith<TextRange>) -> PyVisitorResult {
        walk_stmt_async_with(self, node)
    }

    fn visit_stmt_match(&mut self, node: ast::StmtMatch<TextRange>) -> PyVisitorResult {
        walk_stmt_match(self, node)
    }

    fn visit_stmt_raise(&mut self, node: ast::StmtRaise<TextRange>) -> PyVisitorResult {
        walk_stmt_raise(self, node)
    }

    fn visit_stmt_try(&mut self, node: ast::StmtTry<TextRange>) -> PyVisitorResult {
        walk_stmt_try(self, node)
    }

    fn visit_stmt_try_star(&mut self, node: ast::StmtTryStar<TextRange>) -> PyVisitorResult {
        walk_stmt_try_star(self, node)
    }

    fn visit_stmt_assert(&mut self, node: ast::StmtAssert<TextRange>) -> PyVisitorResult {
        walk_stmt_assert(self, node)
    }

    fn visit_stmt_import(&mut self, node: ast::StmtImport<TextRange>) -> PyVisitorResult {
        walk_stmt_import(self, node)
    }

    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom<TextRange>) -> PyVisitorResult {
        walk_stmt_import_from(self, node)
    }

    fn visit_stmt_global(&mut self, _node: ast::StmtGlobal<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_stmt_nonlocal(&mut self, _node: ast::StmtNonlocal<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_stmt_expr(&mut self, node: ast::StmtExpr<TextRange>) -> PyVisitorResult {
        walk_stmt_expr(self, node)
    }

    fn visit_stmt_pass(&mut self, _node: ast::StmtPass<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_stmt_break(&mut self, _node: ast::StmtBreak<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_stmt_continue(&mut self, _node: ast::StmtContinue<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr<TextRange>) -> PyVisitorResult {
        walk_expr(self, node)
    }

    fn visit_expr_bool_op(&mut self, node: ast::ExprBoolOp<TextRange>) -> PyVisitorResult {
        walk_expr_bool_op(self, node)
    }

    fn visit_expr_named_expr(&mut self, node: ast::ExprNamedExpr<TextRange>) -> PyVisitorResult {
        walk_expr_named_expr(self, node)
    }

    fn visit_expr_bin_op(&mut self, node: ast::ExprBinOp<TextRange>) -> PyVisitorResult {
        walk_expr_bin_op(self, node)
    }

    fn visit_expr_unary_op(&mut self, node: ast::ExprUnaryOp<TextRange>) -> PyVisitorResult {
        walk_expr_unary_op(self, node)
    }

    fn visit_expr_lambda(&mut self, node: ast::ExprLambda<TextRange>) -> PyVisitorResult {
        walk_expr_lambda(self, node)
    }

    fn visit_expr_if_exp(&mut self, node: ast::ExprIfExp<TextRange>) -> PyVisitorResult {
        walk_expr_if_exp(self, node)
    }

    fn visit_expr_dict(&mut self, node: ast::ExprDict<TextRange>) -> PyVisitorResult {
        walk_expr_dict(self, node)
    }

    fn visit_expr_set(&mut self, node: ast::ExprSet<TextRange>) -> PyVisitorResult {
        walk_expr_set(self, node)
    }

    fn visit_expr_list_comp(&mut self, node: ast::ExprListComp<TextRange>) -> PyVisitorResult {
        walk_expr_list_comp(self, node)
    }

    fn visit_expr_set_comp(&mut self, node: ast::ExprSetComp<TextRange>) -> PyVisitorResult {
        walk_expr_set_comp(self, node)
    }

    fn visit_expr_dict_comp(&mut self, node: ast::ExprDictComp<TextRange>) -> PyVisitorResult {
        walk_expr_dict_comp(self, node)
    }

    fn visit_expr_generator_exp(
        &mut self,
        node: ast::ExprGeneratorExp<TextRange>,
    ) -> PyVisitorResult {
        walk_expr_generator_exp(self, node)
    }

    fn visit_expr_await(&mut self, node: ast::ExprAwait<TextRange>) -> PyVisitorResult {
        walk_expr_await(self, node)
    }

    fn visit_expr_yield(&mut self, node: ast::ExprYield<TextRange>) -> PyVisitorResult {
        walk_expr_yield(self, node)
    }

    fn visit_expr_yield_from(&mut self, node: ast::ExprYieldFrom<TextRange>) -> PyVisitorResult {
        walk_expr_yield_from(self, node)
    }

    fn visit_expr_compare(&mut self, node: ast::ExprCompare<TextRange>) -> PyVisitorResult {
        walk_expr_compare(self, node)
    }

    fn visit_expr_call(&mut self, node: ast::ExprCall<TextRange>) -> PyVisitorResult {
        walk_expr_call(self, node)
    }

    fn visit_expr_formatted_value(
        &mut self,
        node: ast::ExprFormattedValue<TextRange>,
    ) -> PyVisitorResult {
        walk_expr_formatted_value(self, node)
    }

    fn visit_expr_joined_str(&mut self, node: ast::ExprJoinedStr<TextRange>) -> PyVisitorResult {
        walk_expr_joined_str(self, node)
    }

    fn visit_expr_constant(&mut self, _node: ast::ExprConstant<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_expr_attribute(&mut self, node: ast::ExprAttribute<TextRange>) -> PyVisitorResult {
        walk_expr_attribute(self, node)
    }

    fn visit_expr_subscript(&mut self, node: ast::ExprSubscript<TextRange>) -> PyVisitorResult {
        walk_expr_subscript(self, node)
    }

    fn visit_expr_starred(&mut self, node: ast::ExprStarred<TextRange>) -> PyVisitorResult {
        walk_expr_starred(self, node)
    }

    fn visit_expr_name(&mut self, _node: ast::ExprName<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_expr_list(&mut self, node: ast::ExprList<TextRange>) -> PyVisitorResult {
        walk_expr_list(self, node)
    }

    fn visit_expr_tuple(&mut self, node: ast::ExprTuple<TextRange>) -> PyVisitorResult {
        walk_expr_tuple(self, node)
    }

    fn visit_expr_slice(&mut self, node: ast::ExprSlice<TextRange>) -> PyVisitorResult {
        walk_expr_slice(self, node)
    }

    fn visit_expr_context(&mut self, _node: ast::ExprContext) -> PyVisitorResult {
        Ok(())
    }

    fn visit_boolop(&mut self, _node: ast::BoolOp) -> PyVisitorResult {
        Ok(())
    }

    fn visit_operator(&mut self, _node: ast::Operator) -> PyVisitorResult {
        Ok(())
    }

    fn visit_unaryop(&mut self, _node: ast::UnaryOp) -> PyVisitorResult {
        Ok(())
    }

    fn visit_cmpop(&mut self, _node: ast::CmpOp) -> PyVisitorResult {
        Ok(())
    }

    fn visit_comprehension(&mut self, _node: ast::Comprehension<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_excepthandler(&mut self, node: ast::ExceptHandler<TextRange>) -> PyVisitorResult {
        walk_excepthandler(self, node)
    }

    fn visit_excepthandler_except_handler(
        &mut self,
        node: ast::ExceptHandlerExceptHandler<TextRange>,
    ) -> PyVisitorResult {
        walk_excepthandler_except_handler(self, node)
    }

    fn visit_arguments(&mut self, _node: ast::Arguments<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_arg(&mut self, _node: ast::Arg<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_keyword(&mut self, _node: ast::Keyword<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_alias(&mut self, _node: ast::Alias<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_withitem(&mut self, _node: ast::WithItem<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_match_case(&mut self, _node: ast::MatchCase<TextRange>) -> PyVisitorResult {
        Ok(())
    }

    fn visit_pattern(&mut self, node: ast::Pattern<TextRange>) -> PyVisitorResult {
        walk_pattern(self, node)
    }

    fn visit_pattern_match_value(
        &mut self,
        node: ast::PatternMatchValue<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_value(self, node)
    }

    fn visit_pattern_match_singleton(
        &mut self,
        _node: ast::PatternMatchSingleton<TextRange>,
    ) -> PyVisitorResult {
        Ok(())
    }

    fn visit_pattern_match_sequence(
        &mut self,
        node: ast::PatternMatchSequence<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_sequence(self, node)
    }

    fn visit_pattern_match_mapping(
        &mut self,
        node: ast::PatternMatchMapping<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_mapping(self, node)
    }

    fn visit_pattern_match_class(
        &mut self,
        node: ast::PatternMatchClass<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_class(self, node)
    }

    fn visit_pattern_match_star(
        &mut self,
        _node: ast::PatternMatchStar<TextRange>,
    ) -> PyVisitorResult {
        Ok(())
    }

    fn visit_pattern_match_as(
        &mut self,
        node: ast::PatternMatchAs<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_as(self, node)
    }

    fn visit_pattern_match_or(
        &mut self,
        node: ast::PatternMatchOr<TextRange>,
    ) -> PyVisitorResult {
        walk_pattern_match_or(self, node)
    }

    fn visit_type_param(&mut self, node: ast::TypeParam<TextRange>) -> PyVisitorResult {
        walk_type_param(self, node)
    }

    fn visit_type_param_type_var(
        &mut self,
        node: ast::TypeParamTypeVar<TextRange>,
    ) -> PyVisitorResult {
        walk_type_param_type_var(self, node)
    }

    fn visit_type_param_param_spec(
        &mut self,
        _node: ast::TypeParamParamSpec<TextRange>,
    ) -> PyVisitorResult {
        Ok(())
    }

    fn visit_type_param_type_var_tuple(
        &mut self,
        _node: ast::TypeParamTypeVarTuple<TextRange>,
    ) -> PyVisitorResult {
        Ok(())
    }

}
