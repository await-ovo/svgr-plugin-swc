#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_common::DUMMY_SP;
use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum NumberOrString {
    String(String),
    Number(f64),
}

#[derive(Deserialize)]
pub struct Options {
    width: Option<NumberOrString>,
    height: Option<NumberOrString>,
}

pub struct SVGEmDimensionsVisitor {
    elements: Vec<String>,
    height_value: JSXAttrValue,
    width_value: JSXAttrValue,
}

impl SVGEmDimensionsVisitor {
    pub fn new(options: Options) -> Self {
        SVGEmDimensionsVisitor {
            elements: vec!["svg".into(), "Svg".into()],
            height_value: get_value(&options.height),
            width_value: get_value(&options.width),
        }
    }

    fn get_attr(&mut self, name: &str) -> JSXAttrOrSpread {
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            name: JSXAttrName::Ident(Ident::new(name.into(), DUMMY_SP)),
            span: DUMMY_SP,
            value: if name == "height" {
                Some(self.height_value.clone())
            } else if name == "width" {
                Some(self.width_value.clone())
            } else {
                None
            },
        })
    }
}

impl VisitMut for SVGEmDimensionsVisitor {
    fn visit_mut_jsx_opening_element(&mut self, jsx_opening_element: &mut JSXOpeningElement) {
        jsx_opening_element.visit_mut_children_with(self);
        if let JSXElementName::Ident(Ident { sym, .. }) = &jsx_opening_element.name {
            if self.elements.contains(&sym.to_string()) {
                let mut replace_width = false;
                let mut replace_height = false;
                for attr in jsx_opening_element.attrs.iter_mut() {
                    if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                        if let JSXAttr {
                            name: JSXAttrName::Ident(Ident { sym, .. }),
                            ..
                        } = jsx_attr
                        {
                            if sym == "height" || sym == "width" {
                                jsx_attr.value = Some(if sym == "height" {
                                    replace_height = true;
                                    self.height_value.clone()
                                } else {
                                    replace_width = true;
                                    self.width_value.clone()
                                });
                            }
                        }
                    }
                }

                if !replace_width {
                    jsx_opening_element.attrs.push(self.get_attr("width"));
                }

                if !replace_height {
                    jsx_opening_element.attrs.push(self.get_attr("height"));
                }
            }
        }
    }
}

fn get_value(raw_option: &Option<NumberOrString>) -> JSXAttrValue {
    if let Some(raw) = raw_option {
        match raw {
            NumberOrString::Number(number_value) => {
                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num((*number_value).into())))),
                })
            }
            NumberOrString::String(string_value) => JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: string_value.as_str().into(),
                raw: None,
            })),
        }
    } else {
        JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "1em".into(),
            raw: None,
        }))
    }
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
        |_| as_folder(SVGEmDimensionsVisitor::new(Options {
            width: None,
            height: None,
        })),
        replace_width_or_height_value,
        // Input codes
        r#"<svg foo="bar" width="100" height="200" />"#,
        // Output codes after transformed with plugin
        r#"<svg foo="bar" width="1em" height="1em" />;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(SVGEmDimensionsVisitor::new(Options {
            width: None,
            height: None,
        })),
        add_attribute_if_it_not_present,
        // Input codes
        r#"<svg foo="bar" />"#,
        // Output codes after transformed with plugin
        r#"<svg foo="bar" width="1em" height="1em" />;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(SVGEmDimensionsVisitor::new(Options {
            width: Some(NumberOrString::Number(24.into())),
            height: Some(NumberOrString::Number(24.into())),
        })),
        accepts_numeric_values,
        // Input codes
        r#"<svg foo="bar" />"#,
        // Output codes after transformed with plugin
        r#"<svg foo="bar" width={24} height={24} />;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(SVGEmDimensionsVisitor::new(Options {
            width: Some(NumberOrString::String("2em".into())),
            height: Some(NumberOrString::String("2em".into())),
        })),
        accepts_string_values,
        // Input codes
        r#"<svg foo="bar" />"#,
        // Output codes after transformed with plugin
        r#"<svg foo="bar" width="2em" height="2em" />;"#
    );
}
