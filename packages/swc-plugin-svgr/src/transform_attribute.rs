use regex::Regex;
use swc_common::DUMMY_SP;
use swc_core::ecma::{ast::*, visit::VisitMut};

pub struct TransformAttributeVisitor;

impl VisitMut for TransformAttributeVisitor {
    fn visit_mut_jsx_attr(&mut self, jsx_attr: &mut JSXAttr) {
        let original_name = jsx_attr.name.clone();
        match original_name {
            JSXAttrName::JSXNamespacedName(JSXNamespacedName {
                ns: Ident { sym: ns, .. },
                name: Ident { sym: name, .. },
                ..
            }) => {
                jsx_attr.name =
                    JSXAttrName::Ident(Ident::new(namespace_to_camel(&ns, &name).into(), DUMMY_SP));
            }
            JSXAttrName::Ident(Ident { sym, .. }) => {
                if sym.eq("class") {
                    jsx_attr.name = JSXAttrName::Ident(Ident::new("className".into(), DUMMY_SP));
                    return;
                }

                if sym.eq("style") {
                    if let Some(JSXAttrValue::Lit(Lit::Str(Str { value, .. }))) =
                        jsx_attr.value.clone()
                    {
                        jsx_attr.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            span: DUMMY_SP,
                            expr: JSXExpr::Expr(Box::new(Expr::Object(css_to_obj(&value)))),
                        }));
                    }
                    return;
                }

                let re = Regex::new(r"^data-|^aria-").unwrap();

                if re.is_match(&sym) {
                    jsx_attr.name =
                        JSXAttrName::Ident(Ident::new(format!("'{}'", sym).into(), DUMMY_SP));
                } else {
                    jsx_attr.name =
                        JSXAttrName::Ident(Ident::new(hyphen_to_camel(&sym).into(), DUMMY_SP));
                }
            }
        };
    }
}

fn namespace_to_camel(ns: &str, name: &str) -> String {
    let new_name = name
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().next().unwrap()
            } else {
                c
            }
        })
        .collect::<String>();

    format!("{}{}", ns, new_name)
}

fn hyphen_to_camel(s: &str) -> String {
    s.split('-')
        .enumerate()
        .map(|(i, part)| {
            if i > 0 {
                let (first, rest) = part.split_at(1);
                first.to_uppercase() + rest
            } else {
                part.to_string()
            }
        })
        .collect()
}

fn css_to_obj(css: &str) -> ObjectLit {
    let mut props: Vec<PropOrSpread> = vec![];

    for el in css.split(';').filter(|el| !el.is_empty()) {
        let mut s = el.split(':');
        if let Some(key) = s.next() {
            let value = s.collect::<Vec<&str>>().join(":").trim().to_owned();
            props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new(hyphen_to_camel(key).into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(if is_numeric(&value) {
                    Lit::Num(Number {
                        span: DUMMY_SP,
                        value: value.parse::<f64>().unwrap(),
                        raw: None,
                    })
                } else {
                    Lit::Str(Str {
                        span: DUMMY_SP,
                        value: value.into(),
                        raw: None,
                    })
                })),
            }))));
        }
    }

    ObjectLit {
        span: DUMMY_SP,
        props,
    }
}

pub fn is_numeric(value: &str) -> bool {
    value.parse::<f64>().is_ok()
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
        |_| as_folder(TransformAttributeVisitor),
        namespace_attribute,
        r#"<svg xmlns:xlink="asdf" />"#,
        r#"<svg xmlnsXlink="asdf" />"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformAttributeVisitor),
        class_attribute,
        r#"<svg class="a b" />"#,
        r#"<svg className="a b" />"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformAttributeVisitor),
        style_attribute,
        r#"<svg style="text-align: center;" />"#,
        r#"<svg style={{
            textAlign: "center",
        }}/>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformAttributeVisitor),
        style_attribute_numeric_value,
        r#"<svg style="font-size: 50;" />"#,
        r#"<svg style={{
            fontSize: 50,
        }}/>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformAttributeVisitor),
        hyphen_to_camel,
        r#"<g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd" />"#,
        r#"<g id="Page-1" stroke="none" strokeWidth="1" fill="none" fillRule="evenodd" />"#
    );
}
