use crate::config::*;
use swc_core::{
    common::{input::StringInput, BytePos, DUMMY_SP},
    ecma::{
        ast::*,
        parser::{EsConfig, Parser, Syntax},
        visit::{VisitMut, VisitMutWith},
    },
    quote,
};

pub mod config;

pub struct TransformSVGComponentVisitor {
    pub config: Config,
    jsx_svg_element: Option<JSXElement>,
}

impl VisitMut for TransformSVGComponentVisitor {
    fn visit_mut_jsx_element(&mut self, jsx_element: &mut JSXElement) {
        jsx_element.visit_mut_children_with(self);
        if let JSXElementName::Ident(tag_name) = &jsx_element.opening.name {
            if *tag_name.sym == *"svg" || *tag_name.sym == *"Svg" {
                self.jsx_svg_element = Some(jsx_element.clone());
            }
        }
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        let mut new_items: Vec<ModuleItem> = vec![];
        let mut import_stmts = self.create_imports();
        let mut export_stmts = self.create_exports();

        new_items.append(&mut import_stmts);

        if let Some(component_body_stmts) = &mut self.create_component_body() {
            new_items.append(component_body_stmts);
        }

        new_items.append(&mut export_stmts);

        module.body = new_items;
    }
}

impl TransformSVGComponentVisitor {
    pub fn new(config: Config) -> Self {
        TransformSVGComponentVisitor {
            config,
            jsx_svg_element: None,
        }
    }

    fn create_imports(&mut self) -> Vec<ModuleItem> {
        let mut imports: Vec<ImportDecl> = Vec::new();
        match self.config.jsx_runtime {
            Some(JSXRuntime::Automatic) => {}
            _ => {
                imports.push(get_jsx_runtime_import(&self.config.jsx_runtime_import));
            }
        };

        if self.config.native {
            get_or_create_import(&mut imports, "react-native-svg")
                .specifiers
                .push(ImportSpecifier::Default(ImportDefaultSpecifier {
                    span: DUMMY_SP,
                    local: Ident::new("Svg".into(), DUMMY_SP),
                }));
        }

        match self.config.expand_props {
            ExpandProps::Boolean(false) => {}
            _ => {
                if self.config.typescript {
                    let (import_source, local) = if self.config.native {
                        ("react-native-svg", "SvgProps")
                    } else {
                        (&*self.config.import_source, "SVGProps")
                    };
                    get_or_create_import(&mut imports, import_source)
                        .specifiers
                        .push(ImportSpecifier::Named(ImportNamedSpecifier {
                            span: DUMMY_SP,
                            local: Ident::new(local.into(), DUMMY_SP),
                            imported: None,
                            is_type_only: true,
                        }));
                }
            }
        }

        if self.config.forward_ref {
            if self.config.typescript {
                get_or_create_import(&mut imports, &self.config.import_source)
                    .specifiers
                    .push(ImportSpecifier::Named(ImportNamedSpecifier {
                        span: DUMMY_SP,
                        local: Ident::new("Ref".into(), DUMMY_SP),
                        imported: None,
                        is_type_only: true,
                    }));
            }
            get_or_create_import(&mut imports, &(self.config.import_source))
                .specifiers
                .push(ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: Ident::new("forwardRef".into(), DUMMY_SP),
                    imported: None,
                    is_type_only: false,
                }));
        }

        if self.config.memo {
            get_or_create_import(&mut imports, &(self.config.import_source))
                .specifiers
                .push(ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: Ident::new("memo".into(), DUMMY_SP),
                    imported: None,
                    is_type_only: false,
                }));
        }

        imports
            .into_iter()
            .map(|decl| ModuleItem::ModuleDecl(ModuleDecl::Import(decl)))
            .collect::<Vec<ModuleItem>>()
    }

    fn create_exports(&mut self) -> Vec<ModuleItem> {
        let mut exports: Vec<ModuleItem> = Vec::new();
        let mut export_identifier = self.config.state.component_name.clone();

        if self.config.forward_ref {
            let stmt = quote!(
                "const ForwardRef = forwardRef($name)" as Stmt,
                name = Ident::new(export_identifier.into(), DUMMY_SP)
            );
            exports.push(ModuleItem::Stmt(stmt));
            export_identifier = "ForwardRef".to_string();
        }

        if self.config.memo {
            exports.push(ModuleItem::Stmt(quote!(
                "const Memo = memo($name)" as Stmt,
                name = Ident::new(export_identifier.into(), DUMMY_SP)
            )));
            export_identifier = "Memo".to_string();
        }

        if self.config.export_type == ExportType::Named
            || !self.config.state.caller.previous_export.is_empty()
        {
            if self.config.named_export.is_empty() {
                panic!("\"namedExport\" not specified");
            }

            exports.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                NamedExport {
                    span: DUMMY_SP,
                    specifiers: vec![ExportSpecifier::Named(ExportNamedSpecifier {
                        span: DUMMY_SP,
                        orig: ModuleExportName::Ident(Ident::new(
                            export_identifier.into(),
                            DUMMY_SP,
                        )),
                        exported: Some(ModuleExportName::Ident(Ident::new(
                            self.config.named_export.clone().into(),
                            DUMMY_SP,
                        ))),
                        is_type_only: false,
                    })],
                    src: None,
                    type_only: false,
                    asserts: None,
                },
            )));

            if !self.config.state.caller.previous_export.is_empty() {
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
                    StringInput::new(
                        &self.config.state.caller.previous_export,
                        BytePos(0),
                        BytePos(0),
                    ),
                    None,
                );

                let mut ast = parser
                    .parse_module()
                    .expect("Failed to parse \"previousExport\"");

                exports.append(&mut ast.body)
            }
        } else {
            exports.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                ExportDefaultExpr {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Ident(Ident::new(export_identifier.into(), DUMMY_SP))),
                },
            )));
        }
        exports
    }

    fn create_component_body(&mut self) -> Option<Vec<ModuleItem>> {
        let (component_props, interfaces) = self.create_component_props();
        match &self.jsx_svg_element {
            Some(jsx_element) => {
                let component_decl = VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(BindingIdent {
                            id: Ident::new(
                                self.config.state.component_name.clone().into(),
                                DUMMY_SP,
                            ),
                            type_ann: None,
                        }),
                        init: Some(Box::new(Expr::Arrow(ArrowExpr {
                            span: DUMMY_SP,
                            params: component_props,
                            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::JSXElement(
                                Box::new(jsx_element.clone()),
                            )))),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                        }))),
                        definite: false,
                    }],
                };

                let mut body_stmts: Vec<ModuleItem> = interfaces
                    .iter()
                    .map(|interface| {
                        ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(Box::new(interface.clone()))))
                    })
                    .collect();

                body_stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(
                    component_decl,
                )))));

                Some(body_stmts)
            }
            None => None,
        }
    }

    fn create_component_props(&mut self) -> (Vec<Pat>, Vec<TsInterfaceDecl>) {
        let mut props: Vec<Pat> = vec![];
        let mut properties: Vec<ObjectPatProp> = vec![];
        let mut interfaces: Vec<TsInterfaceDecl> = vec![];
        let mut property_signatures: Vec<TsTypeElement> = vec![];
        if self.config.title_prop || self.config.desc_prop {
            if self.config.title_prop {
                properties.push(create_object_assign_property("title"));
                properties.push(create_object_assign_property("titleId"));

                if self.config.typescript {
                    property_signatures.push(create_property_signature("title"));
                    property_signatures.push(create_property_signature("titleId"))
                }
            }

            if self.config.desc_prop {
                properties.push(create_object_assign_property("desc"));
                properties.push(create_object_assign_property("descId"));

                if self.config.typescript {
                    property_signatures.push(create_property_signature("desc"));
                    property_signatures.push(create_property_signature("descId"))
                }
            }

            let mut prop = ObjectPat {
                span: DUMMY_SP,
                props: properties,
                optional: false,
                type_ann: None,
            };

            if self.config.typescript {
                interfaces.push(TsInterfaceDecl {
                    span: DUMMY_SP,
                    id: Ident::new("SVGRProps".into(), DUMMY_SP),
                    declare: false,
                    type_params: None,
                    extends: vec![],
                    body: TsInterfaceBody {
                        span: DUMMY_SP,
                        body: property_signatures,
                    },
                });

                prop.type_ann = Some(Box::new(TsTypeAnn {
                    span: DUMMY_SP,
                    type_ann: Box::new(TsType::TsTypeRef(create_type_ref_svgr_props())),
                }));
            }
            props.push(Pat::Object(prop));
        }

        match self.config.expand_props {
            ExpandProps::Boolean(false) => {}
            _ => {
                let props_ident = Ident::new("props".into(), DUMMY_SP);
                if !props.is_empty() && props[0].is_object() {
                    props[0]
                        .as_mut_object()
                        .unwrap()
                        .props
                        .push(ObjectPatProp::Rest(RestPat {
                            span: DUMMY_SP,
                            dot3_token: DUMMY_SP,
                            arg: Box::new(Pat::Ident(BindingIdent {
                                id: props_ident,
                                type_ann: None,
                            })),
                            type_ann: None,
                        }));
                    if self.config.typescript {
                        props[0].as_mut_object().unwrap().type_ann = Some(Box::new(TsTypeAnn {
                            span: DUMMY_SP,
                            type_ann: Box::new(TsType::TsUnionOrIntersectionType(
                                TsUnionOrIntersectionType::TsIntersectionType(TsIntersectionType {
                                    span: DUMMY_SP,
                                    types: vec![
                                        Box::new(TsType::TsTypeRef(create_type_ref_svg_props(
                                            self.config.native,
                                        ))),
                                        Box::new(TsType::TsTypeRef(create_type_ref_svgr_props())),
                                    ],
                                }),
                            )),
                        }));
                    }
                } else {
                    props.push(Pat::Ident(BindingIdent {
                        id: props_ident,
                        type_ann: if self.config.typescript {
                            Some(Box::new(TsTypeAnn {
                                span: DUMMY_SP,
                                type_ann: Box::new(TsType::TsTypeRef(create_type_ref_svg_props(
                                    self.config.native,
                                ))),
                            }))
                        } else {
                            None
                        },
                    }));
                }
            }
        }

        if self.config.forward_ref {
            if props.is_empty() {
                props.push(Pat::Ident(BindingIdent {
                    id: Ident::new("_".into(), DUMMY_SP),
                    type_ann: None,
                }));
            }

            props.push(Pat::Ident(BindingIdent {
                id: Ident::new("ref".into(), DUMMY_SP),
                type_ann: if self.config.typescript {
                    Some(Box::new(TsTypeAnn {
                        span: DUMMY_SP,
                        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                            span: DUMMY_SP,
                            type_name: TsEntityName::Ident(Ident::new("Ref".into(), DUMMY_SP)),
                            type_params: create_svg_svg_element_type_param(),
                        })),
                    }))
                } else {
                    None
                },
            }))
        }

        (props, interfaces)
    }
}

fn get_jsx_runtime_import(cfg: &JSXRuntimeImport) -> ImportDecl {
    let mut specifiers: Vec<ImportSpecifier> = Vec::new();
    if let Some(namespace) = &cfg.namespace {
        specifiers.push(ImportSpecifier::Namespace(ImportStarAsSpecifier {
            span: DUMMY_SP,
            local: Ident::new((**namespace).into(), DUMMY_SP),
        }))
    } else if let Some(default_specifier) = &cfg.default_specifier {
        specifiers.push(ImportSpecifier::Default(ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: Ident::new((**default_specifier).into(), DUMMY_SP),
        }));
    } else if let Some(config_specifiers) = &cfg.specifiers {
        for config_specifier in config_specifiers {
            specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: Ident::new((**config_specifier).into(), DUMMY_SP),
                imported: None,
                is_type_only: false,
            }));
        }
    } else {
        panic!("Specify \"namespace\", \"defaultSpecifier\", or \"specifiers\" in \"jsxRuntimeImport\" option");
    }

    ImportDecl {
        span: DUMMY_SP,
        specifiers,
        src: Box::new(Str {
            span: DUMMY_SP,
            value: cfg.source.clone().into(),
            raw: None,
        }),
        type_only: false,
        asserts: None,
    }
}

fn get_or_create_import<'a>(imports: &'a mut Vec<ImportDecl>, source: &str) -> &'a mut ImportDecl {
    let index = match imports.iter().position(|imp| {
        imp.src.value.eq(source) && !imp.specifiers.iter().any(|s| s.is_namespace())
    }) {
        Some(i) => i,
        None => {
            let imp = ImportDecl {
                span: DUMMY_SP,
                specifiers: Vec::<ImportSpecifier>::new(),
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: source.into(),
                    raw: None,
                }),
                type_only: false,
                asserts: None,
            };

            imports.push(imp);
            imports.len() - 1
        }
    };

    &mut imports[index]
    // }
}

fn create_object_assign_property(key: &str) -> ObjectPatProp {
    ObjectPatProp::Assign(AssignPatProp {
        span: DUMMY_SP,
        key: Ident::new(key.into(), DUMMY_SP),
        value: None,
    })
}

fn create_property_signature(key: &str) -> TsTypeElement {
    TsTypeElement::TsPropertySignature(TsPropertySignature {
        span: DUMMY_SP,
        readonly: false,
        key: Box::new(Expr::Ident(Ident::new(key.into(), DUMMY_SP))),
        computed: false,
        optional: true,
        init: None,
        params: vec![],
        type_ann: Some(Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                span: DUMMY_SP,
                kind: TsKeywordTypeKind::TsStringKeyword,
            })),
        })),
        type_params: None,
    })
}

fn create_type_ref_svg_props(native: bool) -> TsTypeRef {
    let type_name = if native { "SvgProps" } else { "SVGProps" };
    TsTypeRef {
        span: DUMMY_SP,
        type_name: TsEntityName::Ident(Ident::new(type_name.into(), DUMMY_SP)),
        type_params: if native {
            None
        } else {
            create_svg_svg_element_type_param()
        },
    }
}

fn create_type_ref_svgr_props() -> TsTypeRef {
    TsTypeRef {
        span: DUMMY_SP,
        type_name: TsEntityName::Ident(Ident::new("SVGRProps".into(), DUMMY_SP)),
        type_params: None,
    }
}

fn create_svg_svg_element_type_param() -> Option<Box<TsTypeParamInstantiation>> {
    Some(Box::new(TsTypeParamInstantiation {
        span: DUMMY_SP,
        params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident::new("SVGSVGElement".into(), DUMMY_SP)),
            type_params: None,
        }))],
    }))
}
