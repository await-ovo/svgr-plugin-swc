#![allow(clippy::not_unsafe_ptr_arg_deref)]
use swc_core::{
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};



pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}
