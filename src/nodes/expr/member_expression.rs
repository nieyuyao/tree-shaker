use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{
    dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity,
    union::UnionEntity,
  },
  transformer::Transformer,
};
use oxc::ast::ast::{
  ComputedMemberExpression, Expression, MemberExpression, PrivateFieldExpression,
  StaticMemberExpression,
};

const AST_TYPE_READ: AstType2 = AstType2::MemberExpressionRead;
const AST_TYPE_WRITE: AstType2 = AstType2::MemberExpressionWrite;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
  need_optional: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_member_expression_read(&mut self, node: &'a MemberExpression<'a>) -> Entity<'a> {
    let object = self.exec_expression(node.object());

    let indeterminate = if node.optional() {
      match object.test_nullish() {
        Some(true) => return LiteralEntity::new_undefined(),
        Some(false) => false,
        None => true,
      }
    } else {
      false
    };

    if indeterminate {
      self.push_normal_cf_scope(None);
    }

    let key = self.exec_key(node);
    // TODO: handle optional
    let (has_effect, value) = object.get_property(self, &key);

    let data = self.load_data::<Data>(AST_TYPE_READ, node);
    data.has_effect |= has_effect;
    data.need_optional |= indeterminate;

    if indeterminate {
      self.pop_cf_scope();
      UnionEntity::new(vec![value, LiteralEntity::new_undefined()])
    } else {
      value
    }
  }

  pub fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: Entity<'a>,
  ) {
    let dep = self.new_entity_dep(EntityDepNode::MemberExpression(node));
    let value = ForwardedEntity::new(value, dep);

    let object = self.exec_expression(node.object());
    let key = self.exec_key(node);
    let has_effect = object.set_property(self, &key, value);

    let data = self.load_data::<Data>(AST_TYPE_WRITE, node);
    data.has_effect |= has_effect;
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> Entity<'a> {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => {
        LiteralEntity::new_string(node.property.name.as_str())
      }
      MemberExpression::PrivateFieldExpression(node) => {
        LiteralEntity::new_string(node.field.name.as_str())
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_member_expression_read(
    &self,
    node: &'a MemberExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE_READ, node);

    let need_read = need_val || data.has_effect;

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);
        let key = self.transform_expression(expression, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_computed(
            *span,
            object.unwrap(),
            key.unwrap(),
            data.need_optional,
          )))
        } else {
          build_effect!(&self.ast_builder, *span, object, key)
        }
      }
      MemberExpression::StaticMemberExpression(node) => {
        let StaticMemberExpression { span, object, property, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_static(
            *span,
            object.unwrap(),
            property.clone(),
            data.need_optional,
          )))
        } else {
          object
        }
      }
      MemberExpression::PrivateFieldExpression(node) => {
        let PrivateFieldExpression { span, object, field, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);

        if need_read {
          Some(self.ast_builder.expression_member(
            self.ast_builder.member_expression_private_field_expression(
              *span,
              object.unwrap(),
              field.clone(),
              data.need_optional,
            ),
          ))
        } else {
          object
        }
      }
    }
  }

  pub fn transform_member_expression_write(
    &self,
    node: &'a MemberExpression<'a>,
  ) -> Option<MemberExpression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE_WRITE, node);

    let referred = self.is_referred(EntityDepNode::MemberExpression(node));
    let need_write = referred || data.has_effect;

    // TODO: side effect
    need_write.then(|| self.clone_node(node))
  }
}
