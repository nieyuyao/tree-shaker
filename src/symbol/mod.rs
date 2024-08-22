pub mod arguments;

use crate::{
  analyzer::Analyzer,
  entity::{function::FunctionEntity, Entity},
};
use arguments::ArgumentsEntity;
use oxc::{
  ast::ast::{
    AssignmentExpression, BindingRestElement, Class, Expression, FormalParameter, Function,
    UsingDeclaration, VariableDeclarator,
  },
  semantic::SymbolId,
};

#[derive(Debug, Clone, Copy)]
pub enum SymbolSource<'a> {
  VariableDeclarator(&'a VariableDeclarator<'a>, SymbolId),
  Function(&'a Function<'a>),
  ClassDeclaration(&'a Class<'a>),
  UsingDeclaration(&'a UsingDeclaration<'a>),
  FormalParameter(&'a FormalParameter<'a>, SymbolId),
  BindingRestElement(&'a BindingRestElement<'a>, SymbolId),
  Expression(&'a Expression<'a>),
  Assignment(&'a AssignmentExpression<'a>, SymbolId),
  Unknown,
}

impl Default for SymbolSource<'_> {
  fn default() -> Self {
    SymbolSource::Unknown
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn declare_symbol(&mut self, source: SymbolSource<'a>, symbol: SymbolId) {
    self.symbol_source.insert(symbol, source);
  }

  pub(crate) fn get_symbol_source(&self, symbol: SymbolId) -> SymbolSource<'a> {
    *self.symbol_source.get(&symbol).unwrap()
  }

  pub(crate) fn calc_source(&self, source: SymbolSource<'a>) -> Entity {
    match source {
      SymbolSource::VariableDeclarator(node, symbol) => self.calc_variable_declarator(node, symbol),
      SymbolSource::Function(node) => self.calc_function(node),
      SymbolSource::FormalParameter(node, symbol) => self.calc_formal_parameter(node, symbol),
      SymbolSource::BindingRestElement(node, symbol) => {
        self.calc_binding_rest_element(node, symbol).unwrap()
      }
      SymbolSource::Expression(node) => self.calc_expression(node),
      _ => todo!(),
    }
  }

  pub(crate) fn calc_symbol(&self, symbol: SymbolId) -> Entity {
    self.calc_source(self.get_symbol_source(symbol))
  }

  pub(crate) fn read_source(&mut self, source: SymbolSource<'a>) {
    match source {
      SymbolSource::VariableDeclarator(node, symbol) => {
        self.refer_variable_declarator(node, symbol)
      }
      SymbolSource::Function(node) => self.refer_function(node),
      _ => todo!(),
    }
  }

  pub(crate) fn read_symbol(&mut self, symbol: SymbolId) {
    self.read_source(self.get_symbol_source(symbol))
  }

  pub(crate) fn read_symbol_member(&mut self, symbol: SymbolId, member: Entity) -> Entity {
    todo!()
  }

  pub(crate) fn call_symbol(
    &mut self,
    symbol: SymbolId,
    this: Entity,
    args: ArgumentsEntity<'a>,
  ) -> (bool, Entity) {
    let source = self.symbol_source.get(&symbol).expect("Missing declaration");

    match source {
      SymbolSource::Function(node) => self.call_function(node, this, args),
      _ => (true, Entity::Unknown),
    }
  }
}