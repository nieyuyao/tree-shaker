use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityFactory},
  init_object,
};
use oxc::{index::IndexVec, semantic::SymbolId};

#[derive(Debug)]
pub struct ReactContextData<'a> {
  default_value: Entity<'a>,
  stack: Vec<Entity<'a>>,
}

pub type ReactContexts<'a> = IndexVec<SymbolId, ReactContextData<'a>>;

pub fn create_react_create_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    let default_value = args.destruct_as_array(analyzer, dep, 1).0[0];

    let context = analyzer.new_empty_object(&analyzer.builtins.prototypes.object);

    let context_id = analyzer
      .builtins
      .react_data
      .contexts
      .push(ReactContextData { default_value, stack: Vec::new() });

    init_object!(context, {
      "__#internal__context_id" => analyzer.serialize_internal_symbol_id(context_id),
      "Provider" => create_react_context_provider_impl(factory, context_id),
      "Consumer" => create_react_context_consumer_impl(factory, context_id),
    });

    factory.entity(context)
  })
}

fn create_react_context_provider_impl<'a>(
  factory: &'a EntityFactory<'a>,
  context_id: SymbolId,
) -> Entity<'a> {
  factory.implemented_builtin_fn(move |analyzer, dep, _this, args| {
    let props = args.destruct_as_array(analyzer, dep.cloned(), 1).0[0];
    let value = props.get_property(analyzer, dep.cloned(), analyzer.factory.string("value"));

    analyzer.builtins.react_data.contexts[context_id].stack.push(value);

    let children = props.get_property(analyzer, dep, analyzer.factory.string("children"));
    children.consume(analyzer);

    analyzer.builtins.react_data.contexts[context_id].stack.pop();

    analyzer.factory.immutable_unknown
  })
}

fn create_react_context_consumer_impl<'a>(
  factory: &'a EntityFactory<'a>,
  context_id: SymbolId,
) -> Entity<'a> {
  factory.implemented_builtin_fn(move |analyzer, dep, _this, args| {
    let props = args.destruct_as_array(analyzer, dep.cloned(), 1).0[0];
    let value = props.get_property(analyzer, dep.cloned(), analyzer.factory.string("value"));

    analyzer.builtins.react_data.contexts[context_id].stack.push(value);

    let children = props.get_property(analyzer, dep, analyzer.factory.string("children"));
    children.consume(analyzer);

    analyzer.builtins.react_data.contexts[context_id].stack.pop();

    analyzer.factory.immutable_unknown
  })
}

pub fn create_react_use_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(move |analyzer, dep, _this, args| {
    let context_object = args.destruct_as_array(analyzer, box_consumable(()), 1).0[0];
    let context_id = context_object.get_property(
      analyzer,
      box_consumable(()),
      analyzer.factory.string("__#internal__context_id"),
    );
    if let Some(context_id) = analyzer.parse_internal_symbol_id(context_id) {
      let data = &analyzer.builtins.react_data.contexts[context_id];
      let value = data.stack.last().copied().unwrap_or(data.default_value);
      factory.computed(value, dep)
    } else {
      analyzer.thrown_builtin_error("Invalid React context object");
      factory.unknown()
    }
  })
}
