use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::ast::ast::IdentifierReference;

#[derive(Debug, Default, Clone)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    let reference = self.semantic.symbols().get_reference(node.reference_id().unwrap());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      // Known symbol
      if let Some(value) = self.read_symbol(symbol) {
        value
      } else {
        self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
        self.factory.unknown
      }
    } else if node.name == "arguments" {
      // The `arguments` object
      let arguments_consumed = self.consume_arguments(None);
      self.call_scope_mut().need_consume_arguments = !arguments_consumed;
      self.factory.unknown
    } else if let Some(global) = self.builtins.get_global(node.name.as_str()) {
      // Known global
      global
    } else {
      // Unknown global
      if self.config.unknown_global_side_effects {
        self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
        self.refer_to_global();
        self.may_throw();
      }
      self.factory.unknown
    }
  }

  pub fn exec_identifier_reference_write(
    &mut self,
    node: &'a IdentifierReference<'a>,
    value: Entity<'a>,
  ) {
    let dep = box_consumable(AstKind2::IdentifierReference(node));
    let value = self.factory.computed(value, dep);

    let reference = self.semantic.symbols().get_reference(node.reference_id().unwrap());
    debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.write_symbol(symbol, value);
    } else if self.builtins.globals.contains_key(node.name.as_str()) {
      self.add_diagnostic(
        "Should not write to builtin object, it may cause unexpected tree-shaking behavior",
      );
    } else {
      self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
      value.consume(self);
      self.may_throw();
      self.refer_to_global();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_identifier_reference_read(
    &self,
    node: &'a IdentifierReference<'a>,
    need_val: bool,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AstKind2::IdentifierReference(node));

    (data.has_effect || need_val).then(|| self.clone_node(node))
  }

  pub fn transform_identifier_reference_write(
    &self,
    node: &'a IdentifierReference<'a>,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AstKind2::IdentifierReference(node));

    let referred = self.is_referred(AstKind2::IdentifierReference(node));

    (data.has_effect || referred).then(|| self.clone_node(node))
  }
}
