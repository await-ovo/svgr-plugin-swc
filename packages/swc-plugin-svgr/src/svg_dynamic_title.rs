#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_common::DUMMY_SP;
use swc_core::ecma::{ast::*, visit::VisitMut};

#[derive(Deserialize)]
pub struct Options {
    #[serde(default = "default_tag")]
    pub tag: String,
}

fn default_tag() -> String {
    "title".into()
}

pub struct DynamicTitleVisitor {
    elements: Vec<String>,
    options: Options,
}

impl DynamicTitleVisitor {
    pub fn new(options: Options) -> Self {
        DynamicTitleVisitor {
            elements: vec!["svg".into(), "Svg".into()],
            options,
        }
    }

    fn get_tag_element(
        &mut self,
        existing_title_option: &mut Option<JSXElement>,
    ) -> JSXExprContainer {
        let tag_name = &self.options.tag;

        if let Some(existing_title) = existing_title_option {
            existing_title.opening.attrs =
                add_tag_attribute(tag_name.to_string(), existing_title.opening.attrs.clone());
        }

        let conditional_title = Box::new(Expr::Cond(CondExpr {
            span: DUMMY_SP,
            test: Box::new(Expr::Ident(Ident::new(
                tag_name.to_string().into(),
                DUMMY_SP,
            ))),
            cons: Box::new(Expr::JSXElement(Box::new(create_tag_element(
                Ident::new(tag_name.to_string().into(), DUMMY_SP),
                vec![JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident::new(
                        tag_name.to_string().into(),
                        DUMMY_SP,
                    )))),
                }],
                if let Some(existing_title) = existing_title_option {
                    existing_title.opening.attrs.clone()
                } else {
                    vec![create_tag_id_attribute(tag_name.to_string())]
                },
            )))),
            alt: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
        }));

        if let Some(existing_title) = existing_title_option {
            if !existing_title.children.is_empty() {
                return JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Cond(CondExpr {
                        span: DUMMY_SP,
                        test: Box::new(Expr::Bin(BinExpr {
                            span: DUMMY_SP,
                            op: BinaryOp::EqEqEq,
                            left: Box::new(Expr::Ident(Ident::new(
                                tag_name.to_string().into(),
                                DUMMY_SP,
                            ))),
                            right: Box::new(Expr::Ident(Ident::new("undefined".into(), DUMMY_SP))),
                        })),
                        cons: Box::new(Expr::JSXElement(Box::new(existing_title.clone()))),
                        alt: conditional_title,
                    }))),
                };
            }
        }

        JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(conditional_title),
        }
    }
}

impl VisitMut for DynamicTitleVisitor {
    fn visit_mut_jsx_element(&mut self, jsx_element: &mut JSXElement) {
        if let JSXElement {
            opening:
                JSXOpeningElement {
                    name: JSXElementName::Ident(Ident { sym, .. }),
                    ..
                },
            ..
        } = jsx_element
        {
            if self.elements.contains(&sym.to_string()) {
                let mut existing_title_element: Option<JSXElement> = None;

                if let Some(title_element_position) =
                    jsx_element.children.iter().position(|child| {
                        if let JSXElementChild::JSXElement(child_element) = child {
                            if let JSXElement {
                                opening:
                                    JSXOpeningElement {
                                        name: JSXElementName::Ident(Ident { sym, .. }),
                                        ..
                                    },
                                ..
                            } = *(child_element.clone())
                            {
                                if sym == self.options.tag {
                                    existing_title_element = Some(*child_element.clone());
                                    return true;
                                }
                            }
                        }
                        false
                    })
                {
                    jsx_element.children[title_element_position] =
                        JSXElementChild::JSXExprContainer(
                            self.get_tag_element(&mut existing_title_element),
                        );
                } else {
                    jsx_element.children.insert(
                        0,
                        JSXElementChild::JSXExprContainer(
                            self.get_tag_element(&mut existing_title_element),
                        ),
                    )
                }
            }
        }
    }
}

fn add_tag_attribute(tag: String, attributes: Vec<JSXAttrOrSpread>) -> Vec<JSXAttrOrSpread> {
    let mut new_attributes = attributes.clone();
    if let Some(id_position) = attributes.iter().position(|x| {
        if let JSXAttrOrSpread::JSXAttr(JSXAttr {
            name: JSXAttrName::Ident(Ident { sym, .. }),
            ..
        }) = x
        {
            if sym == "id" {
                return true;
            }
        }
        false
    }) {
        match &mut new_attributes[id_position] {
            JSXAttrOrSpread::JSXAttr(jsx_attr) => {
                let id_expr = Box::new(Expr::Ident(Ident::new(
                    format!("{}Id", tag).into(),
                    DUMMY_SP,
                )));
                jsx_attr.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: if let Some(JSXAttrValue::Lit(Lit::Str(existing_str_expr))) =
                        jsx_attr.value.clone()
                    {
                        JSXExpr::Expr(Box::new(Expr::Bin(BinExpr {
                            span: DUMMY_SP,
                            op: BinaryOp::LogicalOr,
                            left: id_expr,
                            right: Box::new(Expr::Lit(Lit::Str(existing_str_expr))),
                        })))
                    } else {
                        JSXExpr::Expr(id_expr)
                    },
                }))
            }
            JSXAttrOrSpread::SpreadElement(_) => {}
        }
    } else {
        new_attributes.push(create_tag_id_attribute(tag));
    }
    new_attributes
}

fn create_tag_id_attribute(tag: String) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident::new("id".into(), DUMMY_SP)),
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident::new(
                format!("{}Id", tag).into(),
                DUMMY_SP,
            )))),
        })),
    })
}

fn create_tag_element(
    tag_ident: Ident,
    children: Vec<JSXExprContainer>,
    attributes: Vec<JSXAttrOrSpread>,
) -> JSXElement {
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            name: JSXElementName::Ident(tag_ident.clone()),
            span: DUMMY_SP,
            attrs: attributes,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(tag_ident),
        }),
        children: children
            .iter()
            .map(|x| JSXElementChild::JSXExprContainer(x.clone()))
            .collect(),
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
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        add_title_attribute_if_not_present,
        r#"<svg></svg>"#,
        r#"<svg>{title ? <title id={titleId}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        add_title_element_and_fallback_to_existing_title,
        r#"<svg><title>Hello</title></svg>"#,
        r#"<svg>{title === undefined ? <title id={titleId}>Hello</title> : title ? <title id={titleId}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        existing_title_contains_jsx_expr,
        r#"<svg><title>{"Hello"}</title></svg>"#,
        r#"<svg>{title === undefined ? <title id={titleId}>{"Hello"}</title> : title ? <title id={titleId}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        preserve_any_existing_title_attributes,
        r#"<svg><title id='a'>Hello</title></svg>"#,
        r#"<svg>{title === undefined ? <title id={titleId || 'a'}>Hello</title> : title ? <title id={titleId || 'a'}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        empty_title,
        r#"<svg><title></title></svg>"#,
        r#"<svg>{title ? <title id={titleId}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options {
            tag: "title".into()
        })),
        self_closing_title,
        r#"<svg><title /></svg>"#,
        r#"<svg>{title ? <title id={titleId}>{title}</title> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        attribute_is_already_present,
        r#"<svg></svg>"#,
        r#"<svg>{desc ? <desc id={descId}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        add_desc_element_and_fallback_to_existing_desc,
        r#"<svg><desc>Hello</desc></svg>"#,
        r#"<svg>{desc === undefined ? <desc id={descId}>Hello</desc> : desc ? <desc id={descId}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        existing_desc_contains_jsx_expr,
        r#"<svg><desc>{"Hello"}</desc></svg>"#,
        r#"<svg>{desc === undefined ? <desc id={descId}>{"Hello"}</desc> : desc ? <desc id={descId}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        preserve_any_existing_desc_attributes,
        r#"<svg><desc id='a'>Hello</desc></svg>"#,
        r#"<svg>{desc === undefined ? <desc id={descId || 'a'}>Hello</desc> : desc ? <desc id={descId || 'a'}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        empty_desc,
        r#"<svg><desc></desc></svg>"#,
        r#"<svg>{desc ? <desc id={descId}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        self_closing_desc,
        r#"<svg><desc /></svg>"#,
        r#"<svg>{desc ? <desc id={descId}>{desc}</desc> : null}</svg>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(DynamicTitleVisitor::new(Options { tag: "desc".into() })),
        desc_attribute_is_already_present,
        r#"<svg><foo /></svg>"#,
        r#"<svg>{desc ? <desc id={descId}>{desc}</desc> : null}<foo /></svg>;"#
    );
}
