#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_common::{DUMMY_SP, input::StringInput, BytePos};
use swc_core::{
    ecma::{
        ast::*,
        visit::{VisitMut, VisitMutWith},
        parser::{EsConfig, Parser, Syntax},
    }
};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum NewValue {
    Boolean(bool),
    Number(f64),
    String(String),
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    value: String,
    new_value: NewValue,
    literal: Option<bool>,
}

#[derive(Deserialize)]
pub struct Options {
    pub values: Vec<Value>,
}

pub struct ReplaceJSXAttributeValueVisitor {
    pub options: Options,
}

impl VisitMut for ReplaceJSXAttributeValueVisitor {
    fn visit_mut_jsx_attr(&mut self, jsx_attr: &mut JSXAttr) {
        jsx_attr.visit_mut_children_with(self);
        if let Some(JSXAttrValue::Lit(Lit::Str(Str { value, .. }))) = &jsx_attr.clone().value {
            for value_option in &self.options.values {
                if value.eq(value_option.value.as_str()) {
                    jsx_attr.value =
                        get_attribute_value(&value_option.new_value, value_option.literal);
                }
            }
        }
    }
}

fn get_attribute_value(new_value: &NewValue, literal_option: Option<bool>) -> Option<JSXAttrValue> {
    match new_value {
        NewValue::String(string_value) => {
            let string_value_literal = Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: string_value.as_str().into(),
                raw: None,
            })));

            if let Some(literal) = literal_option {
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
                            "Failed to parse  newValue \"{}\"",
                            string_value,
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
                    string_value_literal
                }
            } else {
                string_value_literal
            }
        }
        NewValue::Boolean(bool_value) => Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool((*bool_value).into())))),
        })),

        NewValue::Number(number_value) => Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num((*number_value).into())))),
        })),
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
        |_| as_folder(ReplaceJSXAttributeValueVisitor {
            options: Options {
                values: vec![Value {
                    value: "cool".into(),
                    new_value: NewValue::String("not cool".into()),
                    literal: None,
                }]
            }
        }),
        replace_attribute_values,
        // Input codes
        r#"<div something="cool" a={b}/>"#,
        // Output codes after transformed with plugin
        r#"<div something="not cool" a={b}/>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(ReplaceJSXAttributeValueVisitor {
            options: Options {
                values: vec![Value {
                    value: "cool".into(),
                    new_value: NewValue::String("props.color".into()),
                    literal: Some(true),
                }]
            }
        }),
        replace_attribute_values_with_literal,
        // Input codes
        r#"<div something="cool" />"#,
        // Output codes after transformed with plugin
        r#"<div something={props.color} />;"#
    );
}
