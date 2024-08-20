mod context;
mod entity;
mod nodes;
mod utils;

use entity::Entity;
use oxc::{
  allocator::Allocator,
  ast::{
    ast::{Declaration, Program, Statement},
    AstBuilder,
  },
  codegen::{CodeGenerator, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::{Semantic, SemanticBuilder, SymbolId},
  span::{GetSpan, SourceType, Span, SPAN},
};
use rustc_hash::FxHashMap;
use std::{any::Any, mem};

pub(crate) struct TreeShaker<'a> {
  pub sematic: Semantic<'a>,
  pub declaration: FxHashMap<SymbolId, &'a Declaration<'a>>,
  pub current_declaration: Option<&'a Declaration<'a>>,
  pub data: FxHashMap<Span, Box<dyn Any>>,
  pub ast_builder: AstBuilder<'a>,
}

impl<'a> TreeShaker<'a> {
  pub fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    TreeShaker {
      sematic,
      declaration: FxHashMap::default(),
      current_declaration: None,
      data: FxHashMap::default(),
      ast_builder: AstBuilder::new(allocator),
    }
  }

  pub fn execute_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }
  }

  pub fn transform_program(&mut self, ast: &'a mut Program<'a>) -> Program<'a> {
    let Program { span, source_type, hashbang, directives, body: old_statements, .. } =
      mem::replace(
        ast,
        self.ast_builder.program(
          SPAN,
          SourceType::default(),
          None,
          self.ast_builder.vec(),
          self.ast_builder.vec(),
        ),
      );
    let mut new_statements = self.ast_builder.vec::<Statement>();
    for statement in old_statements {
      let new_statement = self.transform_statement(statement);
      if let Some(new_statement) = new_statement {
        new_statements.push(new_statement);
      }
    }
    self.ast_builder.program(span, source_type, hashbang, directives, new_statements)
  }
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn load_data_from_span<D: Default + 'static>(&mut self, span: Span) -> &'a mut D {
    if !self.data.contains_key(&span) {
      self.data.insert(span, Box::new(D::default()));
    }
    let x = self.data.get_mut(&span).unwrap();
    let t = x.downcast_mut::<D>().unwrap();
    unsafe { mem::transmute(t) }
  }

  pub(crate) fn load_data<D: Default + 'static>(&mut self, node: &dyn GetSpan) -> &'a mut D {
    self.load_data_from_span(node.span())
  }

  pub(crate) fn declare_symbol(&mut self, symbol_id: SymbolId) {
    self.current_declaration.map(|declaration| {
      self.declaration.insert(symbol_id, declaration);
    });
  }

  pub(crate) fn read_symbol(&mut self, symbol_id: SymbolId) -> Entity {
    let declaration = self.declaration.get(&symbol_id).expect("Missing declaration");
    self.exec_declaration(declaration, Some(symbol_id)).unwrap()
  }

  pub(crate) fn write_symbol(&mut self, symbol_id: SymbolId, entity: Entity) {
    todo!()
  }
}

pub struct TreeShakeReturn {
  pub minifier_return: MinifierReturn,
  pub codegen_return: CodegenReturn,
}

pub fn tree_shake(source_text: &str) -> TreeShakeReturn {
  let allocator = Allocator::default();
  let source_type = SourceType::default();
  let parser = Parser::new(&allocator, source_text, source_type);
  let ast1 = allocator.alloc(parser.parse().program);
  // TODO: Reuse the AST
  let parser = Parser::new(&allocator, source_text, source_type);
  let ast2 = allocator.alloc(parser.parse().program);
  let sematic_builder = SemanticBuilder::new(source_text, source_type);
  let sematic = sematic_builder.build(&ast1).semantic;
  let mut tree_shaker = TreeShaker::new(&allocator, sematic);

  // Step 1: Execute the program
  tree_shaker.execute_program(ast1);

  // Step 2: Execute exports
  // TODO:

  // Step 3: Remove dead code (transform)
  let mut program = tree_shaker.transform_program(ast2);

  // Step 4: Minify
  let minifier = Minifier::new(MinifierOptions::default());
  let minifier_return = minifier.build(&allocator, &mut program);

  // Step 5: Generate output
  let codegen = CodeGenerator::new();
  let codegen_return = codegen.build(&program);

  TreeShakeReturn { minifier_return, codegen_return }
}
