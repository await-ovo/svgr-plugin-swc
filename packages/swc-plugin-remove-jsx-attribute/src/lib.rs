#![allow(clippy::not_unsafe_ptr_arg_deref)]

use serde::Deserialize;
use swc_core::{
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
#[derive(Deserialize)]
pub struct Options {
    elements: Vec<String>,
    attributes: Vec<String>,
}

pub struct TransformVisitor {
    options: Options,
}

impl VisitMut for TransformVisitor {
    fn visit_mut_jsx_opening_element(&mut self, jsx_opening_element: &mut JSXOpeningElement) {
        if let JSXElementName::Ident(Ident { sym, .. }) = &jsx_opening_element.name {
            if self.options.elements.contains(&sym.to_string()) {
                jsx_opening_element.attrs.retain(|attr| {
                    if let JSXAttrOrSpread::JSXAttr(JSXAttr {
                        name: JSXAttrName::Ident(Ident { sym, .. }),
                        ..
                    }) = attr
                    {
                        !self.options.attributes.contains(&sym.to_string())
                    } else {
                        true
                    }
                })
            }
        }

        jsx_opening_element.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let options = serde_json::from_str::<Options>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for remove-jsx-attribute"),
    )
    .expect("invalid config for remove-jsx-attribute");
    program.fold_with(&mut as_folder(TransformVisitor { options }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::ecma::{
        parser::{EsConfig, Syntax},
        transforms::testing::test,
        visit::as_folder,
    };

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformVisitor {
           options: Options {
            elements: vec!["span".into()],
            attributes: vec!["foo".into()]
           }
        }),
        remove_attributes_from_an_element,
        // Input codes
        r#"<div foo><span foo /></div>"#,
        // Output codes after transformed with plugin
        r#"<div foo><span /></div>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformVisitor {
            options: Options {
             elements: vec!["span".into()],
             attributes: vec!["foo".into()]
            }
         }),
         not_throw_error_when_spread_operator_is_used,
         r#"<div foo><span foo {...props} /></div>"#,
         r#"<div foo><span {...props} /></div>;"#
    );
}