#![allow(clippy::not_unsafe_ptr_arg_deref)]

use swc_core::{
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use transform_svg_component::{ config::Config, TransformSVGComponentVisitor};

#[plugin_transform]
pub fn svg_to_component(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Config>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config for transform-svg-component"),
    )
    .expect("invalid config for transform-svg-component");

    program.fold_with(&mut as_folder(TransformSVGComponentVisitor::new(config)))
}
