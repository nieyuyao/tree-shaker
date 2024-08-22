use super::Entity;
use crate::{symbol::arguments::ArgumentsEntity, Analyzer};
use oxc::span::Span;

#[derive(Debug, Default, Clone)]
pub struct FunctionEntity {
  span: Span,
}

impl FunctionEntity {
  pub fn new(span: Span) -> Self {
    FunctionEntity { span }
  }

  pub(crate) fn call<'a>(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: Entity,
    args: ArgumentsEntity<'a>,
  ) -> (bool, Entity) {
    let node = analyzer.functions.get(&self.span).unwrap();
    analyzer.call_function(node, this, args)
  }
}