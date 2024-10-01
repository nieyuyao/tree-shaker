use oxc::{
  allocator::Allocator, codegen::CodegenOptions, minifier::MinifierOptions, span::SourceType,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct Result {
  pub output: String,
  pub diagnostics: Vec<String>,
}

#[wasm_bindgen]
pub fn tree_shake(input: String, do_tree_shake: bool, do_minify: bool, eval_mode: bool) -> Result {
  let result = tree_shake::tree_shake(tree_shake::TreeShakeOptions {
    config: tree_shake::TreeShakeConfig::default(),
    allocator: &Allocator::default(),
    source_type: SourceType::default(),
    source_text: input,
    tree_shake: do_tree_shake,
    minify: do_minify.then(|| MinifierOptions::default()),
    code_gen: CodegenOptions { single_quote: true, minify: do_minify },
    eval_mode,
  });
  Result {
    output: result.codegen_return.source_text,
    diagnostics: result.diagnostics.into_iter().collect(),
  }
}
