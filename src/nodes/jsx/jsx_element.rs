use crate::{analyzer::Analyzer, build_effect, entity::Entity, transformer::Transformer};
use oxc::ast::{
  ast::{Expression, JSXClosingElement, JSXElement, JSXOpeningElement, PropertyKind},
  NONE,
};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_element(&mut self, node: &'a JSXElement<'a>) -> Entity<'a> {
    let tag = self.exec_jsx_element_name(&node.opening_element.name);
    let attributes = self.exec_jsx_attributes(&node.opening_element.attributes);
    let children = self.exec_jsx_children(&node.children);
    attributes.init_property(
      self,
      PropertyKind::Init,
      self.factory.string("children"),
      children,
      true,
    );
    tag.jsx(self, self.factory.entity(attributes))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_element(
    &self,
    node: &'a JSXElement<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val {
      Some(self.ast_builder.expression_from_jsx_element(self.transform_jsx_element_need_val(node)))
    } else {
      self.transform_jsx_element_effect_only(node)
    }
  }

  pub fn transform_jsx_element_effect_only(
    &self,
    node: &'a JSXElement<'a>,
  ) -> Option<Expression<'a>> {
    let JSXElement { span, opening_element, children, .. } = node;

    build_effect!(
      self.ast_builder,
      *span,
      vec![self.transform_jsx_element_name_effect_only(&opening_element.name)],
      self.transform_jsx_attributes_effect_only(&opening_element.attributes),
      self.transform_jsx_children_effect_only(children),
    )
  }

  pub fn transform_jsx_element_need_val(&self, node: &'a JSXElement<'a>) -> JSXElement<'a> {
    let JSXElement { span, opening_element, closing_element, children } = node;

    let name = self.transform_jsx_element_name_need_val(&opening_element.name);

    let closing_element = closing_element.as_ref().map(|closing_element| {
      let JSXClosingElement { span, .. } = closing_element.as_ref();

      self.ast_builder.jsx_closing_element(*span, self.clone_node(&name))
    });

    self.ast_builder.jsx_element(
      *span,
      {
        let JSXOpeningElement { span, self_closing, attributes, .. } = opening_element.as_ref();

        self.ast_builder.jsx_opening_element(
          *span,
          *self_closing,
          name,
          self.transform_jsx_attributes_need_val(attributes),
          NONE,
        )
      },
      closing_element,
      self.transform_jsx_children_need_val(children),
    )
  }
}