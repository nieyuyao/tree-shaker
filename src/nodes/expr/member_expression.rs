use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{dep::EntityDep, entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{
  AssignmentTarget, ComputedMemberExpression, Expression, MemberExpression, StaticMemberExpression,
};

const AST_TYPE: AstType2 = AstType2::MemberExpression;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_member_expression_read(
    &mut self,
    node: &'a MemberExpression<'a>,
  ) -> Entity<'a> {
    let object = self.exec_expression(node.object());
    let key = self.exec_key(node);
    // TODO: handle optional
    let (has_effect, value) = object.get_property(self, &key);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    value
  }

  pub(crate) fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: Entity<'a>,
    dep: EntityDep<'a>,
  ) {
    let object = self.exec_expression(node.object());
    let key = self.exec_key(node);
    let has_effect = object.set_property(self, &key, ForwardedEntity::new(value, dep));

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> Entity<'a> {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => {
        LiteralEntity::new_string(node.property.name.as_str())
      }
      MemberExpression::PrivateFieldExpression(node) => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_member_expression_read(
    &mut self,
    node: MemberExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let need_read = need_val || data.has_effect;

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, optional, .. } = node.unbox();

        let object = self.transform_expression(object, need_read);
        let key = self.transform_expression(expression, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_computed(
            span,
            object.unwrap(),
            key.unwrap(),
            optional,
          )))
        } else {
          build_effect!(&self.ast_builder, span, object, key)
        }
      }
      MemberExpression::StaticMemberExpression(node) => {
        let StaticMemberExpression { span, object, property, optional, .. } = node.unbox();

        let object = self.transform_expression(object, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_static(
            span,
            object.unwrap(),
            property,
            optional,
          )))
        } else {
          object
        }
      }
      MemberExpression::PrivateFieldExpression(node) => todo!(),
    }
  }

  pub(crate) fn transform_member_expression_write(
    &mut self,
    node: MemberExpression<'a>,
    need_write: bool,
  ) -> Option<AssignmentTarget<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let need_write = need_write || data.has_effect;

    // TODO: side effect
    need_write.then(|| {
      self
        .ast_builder
        .assignment_target_simple(self.ast_builder.simple_assignment_target_member_expression(node))
    })
  }
}