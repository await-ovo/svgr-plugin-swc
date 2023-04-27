#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_core::{
    ecma::{ast::*, visit::VisitMutWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

use crate::transform_attribute::TransformAttributeVisitor;
use add_jsx_attribute::{AddJSXAttributeVisitor, Options as AddJSXAttributeOptions};
use remove_jsx_attribute::{Options as RemoveAttributeOptions, RemoveJSXAttributeVisitor};
use remove_jsx_empty_expression::RemoveEmptyExpressionVisitor;
use svg_em_dimensions::{Options as SVGEmDimensionsOptions, SVGEmDimensionsVisitor};
use transform_svg_component::{
    config::Config as TransformSVGComponentOptions, TransformSVGComponentVisitor,
};

use replace_jsx_attribute_value::{
    Options as ReplaceAttributeValueOptions, ReplaceJSXAttributeValueVisitor,
};
use svg_dynamic_title::{Options as DynamicTitleOptions, DynamicTitleVisitor};
use transform_react_native_svg::{
    TransformReactNativeSVGVisitor
};

pub mod add_jsx_attribute;
pub mod remove_jsx_attribute;
pub mod remove_jsx_empty_expression;
pub mod replace_jsx_attribute_value;
pub mod svg_dynamic_title;
pub mod svg_em_dimensions;
pub mod transform_attribute;
pub mod transform_react_native_svg;

#[derive(Deserialize)]
pub struct Options {
    transform_svg_component: Option<TransformSVGComponentOptions>,
    em_dimensions: Option<SVGEmDimensionsOptions>,
    remove_jsx_attribute: Option<RemoveAttributeOptions>,
    add_jsx_attribute: Option<AddJSXAttributeOptions>,
    replace_attribute_values: Option<ReplaceAttributeValueOptions>,
    title_prop: bool,
    desc_prop: bool,
    native: bool,
}

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let options = serde_json::from_str::<Options>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for swc-plugin-svgr"),
    )
    .expect("invalid config for remove-jsx-attribute");

    if let Some(transform_svg_component_options) = options.transform_svg_component {
        program.visit_mut_with(&mut TransformSVGComponentVisitor::new(
            transform_svg_component_options,
        ));
    }

    if let Some(em_dimensions_options) = options.em_dimensions {
        program.visit_mut_with(&mut SVGEmDimensionsVisitor::new(em_dimensions_options));
    }

    if let Some(remove_attribute_options) = options.remove_jsx_attribute {
        program.visit_mut_with(&mut RemoveJSXAttributeVisitor {
            options: remove_attribute_options,
        });
    }

    if let Some(add_attribute_options) = options.add_jsx_attribute {
        program.visit_mut_with(&mut AddJSXAttributeVisitor {
            options: add_attribute_options,
        });
    }

    program.visit_mut_with(&mut RemoveEmptyExpressionVisitor);

    if let Some(replace_attribute_values_options) = options.replace_attribute_values {
        program.visit_mut_with(&mut ReplaceJSXAttributeValueVisitor {
            options: replace_attribute_values_options,
        });
    }

    if options.title_prop {
        program.visit_mut_with(&mut DynamicTitleVisitor::new(DynamicTitleOptions {
            tag: "title".into(),
        }));
    }

    if options.desc_prop {
        program.visit_mut_with(&mut DynamicTitleVisitor::new(DynamicTitleOptions {
            tag: "desc".into(),
        }));
    }

    if options.native {
        program
            .visit_mut_with(&mut TransformReactNativeSVGVisitor {comments: metadata.comments});
    }

    program.visit_mut_with(&mut TransformAttributeVisitor);

    program
}
