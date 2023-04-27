#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_core::{
    common::{input::StringInput, BytePos, DUMMY_SP},
    ecma::{
        ast::*,
        parser::{EsConfig, Parser, Syntax},
        visit::{VisitMut, VisitMutWith},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Position {
    Start,
    End,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Null,
}

#[derive(Deserialize)]
struct AttributeOption {
    name: String,
    value: Option<Value>,
    spread: Option<bool>,
    literal: Option<bool>,
    position: Option<Position>,
}

#[derive(Deserialize)]
pub struct Options {
    elements: Vec<String>,
    attributes: Vec<AttributeOption>,
}

pub struct AddJSXAttributeVisitor {
    pub options: Options,
}

impl VisitMut for AddJSXAttributeVisitor {
    fn visit_mut_jsx_opening_element(&mut self, jsx_opening_element: &mut JSXOpeningElement) {
        jsx_opening_element.visit_mut_children_with(self);

        if let JSXElementName::Ident(Ident { sym, .. }) = &jsx_opening_element.name {
            if self.options.elements.contains(&sym.to_string()) {
                for attribute_option in &self.options.attributes {
                    let new_attribute = get_attribute(attribute_option);

                    if let Some(index) = jsx_opening_element
                        .attrs
                        .iter()
                        .position(|x| is_equal_with_new_attribute(x.clone(), attribute_option))
                    {
                        jsx_opening_element.attrs[index] = new_attribute;
                    } else {
                        match attribute_option.position {
                            Some(Position::End) | None => {
                                jsx_opening_element.attrs.push(new_attribute);
                            }
                            Some(Position::Start) => {
                                jsx_opening_element.attrs.insert(0, new_attribute);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_attribute(attribute_option: &AttributeOption) -> JSXAttrOrSpread {
    let AttributeOption { name, spread, .. } = attribute_option;

    if let Some(is_spread) = spread {
        if *is_spread {
            return JSXAttrOrSpread::SpreadElement(SpreadElement {
                dot3_token: DUMMY_SP,
                expr: Box::new(Expr::Ident(Ident::new(name.to_string().into(), DUMMY_SP))),
            });
        }
    }

    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident::new(name.to_string().into(), DUMMY_SP)),
        value: get_attribute_value(attribute_option),
    })
}

fn get_attribute_value(option: &AttributeOption) -> Option<JSXAttrValue> {
    if let Some(value) = &option.value {
        match value {
            Value::Boolean(bool_value) => Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                    span: DUMMY_SP,
                    value: *bool_value,
                })))),
            })),
            Value::Number(number_value) => Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: *number_value,
                    raw: None,
                })))),
            })),
            Value::String(string_value) => {
                let str_lit_attribute = Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: (*string_value).clone().into(),
                    raw: None,
                })));
                if let Some(literal) = option.literal {
                    if literal {
                        let mut parser = Parser::new(
                            Syntax::Es(EsConfig {
                                jsx: false,
                                fn_bind: false,
                                decorators: false,
                                decorators_before_export: false,
                                export_default_from: true,
                                import_assertions: false,
                                allow_return_outside_function: false,
                                allow_super_outside_method: false,
                                auto_accessors: false,
                            }),
                            StringInput::new(string_value, BytePos(0), BytePos(0)),
                            None,
                        );

                        let ast = parser.parse_module().unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse attribute value \"{}\" expression",
                                string_value
                            )
                        });

                        if let ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) = &ast.body[0] {
                            Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                span: DUMMY_SP,
                                expr: JSXExpr::Expr(expr.clone()),
                            }))
                        } else {
                            None
                        }
                    } else {
                        str_lit_attribute
                    }
                } else {
                    str_lit_attribute
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn is_equal_with_new_attribute(
    attribute: JSXAttrOrSpread,
    attribute_option: &AttributeOption,
) -> bool {
    let mut is_equal_attribute = false;

    if let Some(spread) = attribute_option.spread {
        if spread {
            if let JSXAttrOrSpread::SpreadElement(SpreadElement { expr, .. }) = attribute {
                if let Expr::Ident(Ident { sym, .. }) = *expr {
                    is_equal_attribute = sym == attribute_option.name;
                }
            }
        }
    } else if let JSXAttrOrSpread::JSXAttr(JSXAttr {
        name: JSXAttrName::Ident(Ident { sym, .. }),
        ..
    }) = attribute
    {
        is_equal_attribute = sym == attribute_option.name;
    }

    is_equal_attribute
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
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["div".into()],
                attributes: vec![AttributeOption {
                    name: "disabled".into(),
                    spread: None,
                    literal: None,
                    value: None,
                    position: None,
                }]
            }
        }),
        add_simple_attribute,
        // Input codes
        r#"<div />"#,
        // Output codes after transformed with plugin
        r#"<div disabled />;"#
    );
    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["div".into()],
                attributes: vec![AttributeOption {
                    name: "disabled".into(),
                    spread: None,
                    literal: None,
                    value: Some(Value::String("true".into())),
                    position: None,
                }]
            }
        }),
        add_attribute_with_value,
        // Input codes
        r#"<div />"#,
        // Output codes after transformed with plugin
        r#"<div disabled="true" />;"#
    );
    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["div".into()],
                attributes: vec![AttributeOption {
                    name: "ref".into(),
                    spread: None,
                    literal: Some(true),
                    value: Some(Value::String("ref".into())),
                    position: None,
                }]
            }
        }),
        add_literal_attribute,
        // Input codes
        r#"<div />"#,
        // Output codes after transformed with plugin
        r#"<div ref={ref} />;"#
    );
    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["div".into()],
                attributes: vec![AttributeOption {
                    name: "props".into(),
                    spread: Some(true),
                    literal: None,
                    value: None,
                    position: Some(Position::Start),
                }]
            }
        }),
        add_spread_attribute_start,
        // Input codes
        r#"<div foo><span /></div>"#,
        // Output codes after transformed with plugin
        r#"<div {...props} foo><span /></div>;"#
    );
    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["span".into()],
                attributes: vec![AttributeOption {
                    name: "props".into(),
                    spread: Some(true),
                    literal: None,
                    value: None,
                    position: Some(Position::End),
                }]
            }
        }),
        add_spread_attribute_end,
        // Input codes
        r#"<div><span foo="bar" /></div>"#,
        // Output codes after transformed with plugin
        r#"<div><span foo="bar" {...props} /></div>;"#
    );
    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(AddJSXAttributeVisitor {
            options: Options {
                elements: vec!["div".into()],
                attributes: vec![AttributeOption {
                    name: "disabled".into(),
                    spread: None,
                    literal: None,
                    value: Some(Value::Boolean(false)),
                    position: None,
                }]
            }
        }),
        replace_attribute,
        // Input codes
        r#"<div disabled />"#,
        // Output codes after transformed with plugin
        r#"<div disabled={false} />;"#
    );
}
