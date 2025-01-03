use crate::{analyzer::Analyzer, build_effect, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, ImportExpression};

impl<'a> Analyzer<'a> {
  pub fn exec_import_expression(&mut self, node: &'a ImportExpression<'a>) -> Entity<'a> {
    let mut deps = vec![];

    deps.push(self.exec_expression(&node.source).get_to_string(self));

    for argument in &node.arguments {
      deps.push(self.exec_expression(argument));
    }

    // FIXME: if have side effects, then consume all deps

    self.factory.computed_unknown(self.consumable(deps))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_import_expression(
    &self,
    node: &'a ImportExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ImportExpression { span, source, arguments, phase } = node;

    // FIXME: side effects
    let need_import = need_val;

    let source = self.transform_expression(source, need_import);

    if need_import {
      let mut transformed_arguments = self.ast_builder.vec();
      for argument in arguments {
        transformed_arguments.push(self.transform_expression(argument, true).unwrap());
      }
      Some(self.ast_builder.expression_import(
        *span,
        source.unwrap(),
        transformed_arguments,
        *phase,
      ))
    } else {
      let mut effects = vec![source];
      for argument in arguments {
        effects.push(self.transform_expression(argument, false));
      }
      build_effect!(&self.ast_builder, *span, effects)
    }
  }
}
