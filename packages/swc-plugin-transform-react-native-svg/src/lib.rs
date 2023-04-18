#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
use swc_common::{
    comments::{Comment, CommentKind, Comments},
    DUMMY_SP,
};
use swc_core::{
    ecma::{
        ast::*,
        visit::{VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub struct State {
    replaced_components: HashSet<String>,
    unsupported_components: HashSet<String>,
    element_to_components: HashMap<String, String>,
}

impl Default for State {
    fn default() -> Self {
        State {
            replaced_components: HashSet::new(),
            unsupported_components: HashSet::new(),
            element_to_components: [
                ("svg", "Svg"),
                ("circle", "Circle"),
                ("clipPath", "ClipPath"),
                ("ellipse", "Ellipse"),
                ("g", "G"),
                ("linearGradient", "LinearGradient"),
                ("radialGradient", "RadialGradient"),
                ("line", "Line"),
                ("path", "Path"),
                ("pattern", "Pattern"),
                ("polygon", "Polygon"),
                ("polyline", "Polyline"),
                ("rect", "Rect"),
                ("symbol", "Symbol"),
                ("text", "Text"),
                ("textPath", "TextPath"),
                ("tspan", "TSpan"),
                ("use", "Use"),
                ("defs", "Defs"),
                ("stop", "Stop"),
                ("mask", "Mask"),
                ("image", "Image"),
                ("foreignObject", "ForeignObject"),
            ]
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect(),
        }
    }
}

pub struct ImportDeclVisitor<C: Comments> {
    state: Rc<RefCell<State>>,
    comments: C,
}

impl<C: Comments> ImportDeclVisitor<C> {
    pub fn new(state: Rc<RefCell<State>>, comments: C) -> Self {
        ImportDeclVisitor { state, comments }
    }
}

pub struct JSXElementVisitor {
    state: Rc<RefCell<State>>,
    is_in_svg_element: bool,
}

impl JSXElementVisitor {
    fn new(state: Rc<RefCell<State>>) -> Self {
        JSXElementVisitor {
            is_in_svg_element: false,
            state,
        }
    }
}

impl VisitMut for JSXElementVisitor {
    fn visit_mut_jsx_element(&mut self, jsx_element: &mut JSXElement) {
        let old_is_in_svg_element = self.is_in_svg_element;
        if let JSXElement {
            opening:
                JSXOpeningElement {
                    name: JSXElementName::Ident(Ident { sym, .. }),
                    ..
                },
            ..
        } = jsx_element
        {
            if sym == "svg" {
                self.is_in_svg_element = true;
            }

            if self.is_in_svg_element {
                jsx_element.children.retain(|x| {
                    if let JSXElementChild::JSXElement(child_element) = x {
                        let mut state = self.state.borrow_mut();
                        if let JSXElement {
                            opening:
                                JSXOpeningElement {
                                    name:
                                        JSXElementName::Ident(Ident {
                                            sym: child_element_name,
                                            ..
                                        }),
                                    ..
                                },
                            ..
                        } = *(child_element.clone())
                        {
                            if !state
                                .element_to_components
                                .contains_key(&child_element_name.to_string())
                            {
                                state
                                    .unsupported_components
                                    .insert(child_element_name.to_string());
                                return false;
                            }
                        }
                    }
                    true
                });

                let component = {
                    let state = self.state.borrow();
                    state.element_to_components.get(&sym.to_string()).cloned()
                };

                if let Some(component) = component {
                    let new_name =
                        JSXElementName::Ident(Ident::new(component.as_str().into(), DUMMY_SP));
                    jsx_element.opening.name = new_name.clone();
                    if let Some(closing_element) = &mut jsx_element.closing {
                        if let JSXElementName::Ident(Ident { .. }) = &mut closing_element.name {
                            closing_element.name = new_name;
                        }
                    }
                    self.state
                        .borrow_mut()
                        .replaced_components
                        .insert(component);
                }
            }
        }

        jsx_element.visit_mut_children_with(self);

        self.is_in_svg_element = old_is_in_svg_element;
    }
}

impl<C: Comments> VisitMut for ImportDeclVisitor<C> {
    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        let Str {
            value: source_value,
            ..
        } = *(import_decl.src.clone());
        if source_value.to_string() == "react-native-svg" {
            for component in &self.state.borrow().replaced_components {
                if import_decl.specifiers.iter().any(|x| match x {
                    ImportSpecifier::Default(ImportDefaultSpecifier {
                        local: Ident { sym, .. },
                        ..
                    }) => sym == component,
                    ImportSpecifier::Named(ImportNamedSpecifier {
                        local: Ident { sym, .. },
                        ..
                    }) => sym == component,

                    ImportSpecifier::Namespace(ImportStarAsSpecifier {
                        local: Ident { sym, .. },
                        ..
                    }) => sym == component,
                }) {
                    continue;
                }
                import_decl
                    .specifiers
                    .push(ImportSpecifier::Named(ImportNamedSpecifier {
                        span: DUMMY_SP,
                        local: Ident::new(component.to_string().into(), DUMMY_SP),
                        imported: None,
                        is_type_only: false,
                    }))
            }
        } else {
            return;
        }

        if !self.state.borrow().unsupported_components.is_empty() {
            let comment_list = self
                .state
                .borrow()
                .unsupported_components
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>()
                .join(",");

            self.comments.add_trailing(
                import_decl.span.hi,
                Comment {
                    kind: CommentKind::Block,
                    span: DUMMY_SP,
                    text: format!(
                        " SVGR has dropped some elements not supported by react-native-svg: {} ",
                        comment_list
                    )
                    .into(),
                },
            )
        }
    }
}

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let state: Rc<RefCell<State>> = Rc::new(RefCell::new(State::default()));

    program.visit_mut_with(&mut (JSXElementVisitor::new(state.clone())));
    program.visit_mut_with(&mut (ImportDeclVisitor::new(state, metadata.comments)));

    program
}
