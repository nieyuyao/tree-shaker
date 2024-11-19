use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityFactory},
};

pub fn create_react_create_element_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    let (args, children, _) = args.destruct_as_array(analyzer, dep, 2);
    let [tag, props] = args[..] else { unreachable!() };
    let props = match props.test_nullish() {
      Some(true) => {
        analyzer.factory.entity(analyzer.new_empty_object(&analyzer.builtins.prototypes.object))
      }
      Some(false) => props,
      None => analyzer.factory.union((
        props,
        analyzer.factory.entity(analyzer.new_empty_object(&analyzer.builtins.prototypes.object)),
      )),
    };
    props.set_property(analyzer, box_consumable(()), analyzer.factory.string("children"), children);
    analyzer.factory.react_element(tag, props)
  })
}
