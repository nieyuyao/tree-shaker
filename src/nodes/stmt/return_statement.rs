use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, forwarded::ForwardedEntity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{ReturnStatement, Statement};

impl<'a> Analyzer<'a> {
  pub fn exec_return_statement(&mut self, node: &'a ReturnStatement) {
    let value = node
      .argument
      .as_ref()
      .map_or_else(|| LiteralEntity::new_undefined(), |expr| self.exec_expression(expr));
    let dep = self.new_entity_dep(EntityDepNode::ReturnStatement(node));
    let value = ForwardedEntity::new(value, dep);

    let call_scope = self.call_scope_mut();
    call_scope.returned_values.push(value);
    let cf_scope_id = call_scope.cf_scope_index;
    self.exit_to(cf_scope_id);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_return_statement(&self, node: &'a ReturnStatement<'a>) -> Option<Statement<'a>> {
    let need_val = self.is_referred(EntityDepNode::ReturnStatement(&node));

    let ReturnStatement { span, argument } = node;

    Some(self.ast_builder.statement_return(
      *span,
      argument.as_ref().and_then(|arg| self.transform_expression(arg, need_val)),
    ))
  }
}
