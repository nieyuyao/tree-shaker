use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, entity::Entity, function::FunctionEntity},
  scope::variable_scope::VariableScopes,
  transformer::Transformer,
};
use oxc::ast::ast::{
  ArrowFunctionExpression, Expression, TSTypeAnnotation, TSTypeParameterDeclaration,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
  ) -> Entity<'a> {
    let dep = self.new_entity_dep(EntityDepNode::ArrowFunctionExpression(node));
    FunctionEntity::new(dep.clone(), self.scope_context.variable_scopes.clone())
  }

  pub fn call_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
    variable_scopes: Rc<VariableScopes<'a>>,
    args: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.push_call_scope(variable_scopes, self.call_scope().this.clone(), node.r#async, false);

    self.exec_formal_parameters(&node.params, args);
    if node.expression {
      self.exec_function_expression_body(&node.body);
    } else {
      self.exec_function_body(&node.body);
    }

    self.pop_call_scope()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_arrow_function_expression(
    &self,
    node: &'a ArrowFunctionExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val || self.is_referred(EntityDepNode::ArrowFunctionExpression(&node)) {
      let ArrowFunctionExpression { span, expression, r#async, params, body, .. } = node;

      let params = self.transform_formal_parameters(params);
      let body = if *expression {
        self.transform_function_expression_body(body)
      } else {
        self.transform_function_body(body)
      };

      Some(self.ast_builder.expression_arrow_function(
        *span,
        *expression,
        *r#async,
        None::<TSTypeParameterDeclaration>,
        params,
        None::<TSTypeAnnotation>,
        body,
      ))
    } else {
      None
    }
  }
}
