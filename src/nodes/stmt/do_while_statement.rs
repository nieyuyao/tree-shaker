use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  ast::ast::{DoWhileStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::DoWhileStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_test: bool,
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_do_while_statement(&mut self, node: &'a DoWhileStatement<'a>) {
    let data = self.load_data::<Data>(AST_TYPE, node);

    // Execute the first round.
    let cf_scope_id = self.push_loop_or_switch_cf_scope(Some(false));
    self.push_variable_scope(cf_scope_id);

    self.exec_statement(&node.body);

    if self.cf_scope().must_exited() {
      return;
    }

    data.need_test = true;
    let test = self.exec_expression(&node.test);

    self.pop_variable_scope();
    self.pop_cf_scope();

    // The rest is the same as while statement.
    if test.test_truthy() == Some(false) {
      return;
    }
    test.consume_self(self);

    data.need_loop = true;

    let cf_scope_id = self.push_loop_or_switch_cf_scope(None);
    self.push_variable_scope(cf_scope_id);

    self.exec_statement(&node.body);
    self.exec_expression(&node.test).consume_self(self);

    self.pop_variable_scope();
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_do_while_statement(
    &self,
    node: &'a DoWhileStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let DoWhileStatement { span, test, body, .. } = node;
    let body_span = body.span();

    let body = self.transform_statement(body);

    if !data.need_test {
      body
    } else {
      let test = self.transform_expression(test, data.need_loop);
      if !data.need_loop {
        match (body, test) {
          (Some(body), Some(test)) => {
            let mut statements = self.ast_builder.vec();
            statements.push(body);
            statements.push(self.ast_builder.statement_expression(*span, test));
            Some(self.ast_builder.statement_block(*span, statements))
          }
          (None, Some(test)) => Some(self.ast_builder.statement_expression(*span, test)),
          (Some(body), None) => Some(body),
          (None, None) => None,
        }
      } else {
        Some(self.ast_builder.statement_do_while(
          *span,
          body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
          test.unwrap(),
        ))
      }
    }
  }
}