#![allow(clippy::not_unsafe_ptr_arg_deref)]
use swc_core::{
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_jsx_element(&mut self, jsx_element: &mut JSXElement) {
        jsx_element.visit_mut_children_with(self);

        jsx_element.children.retain(|element| {
            !matches!(element, JSXElementChild::JSXExprContainer(JSXExprContainer { expr: JSXExpr::JSXEmptyExpr(_), .. }))
        })
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
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
        |_| as_folder(TransformVisitor),
        remove_empty_expression,
        // Input codes
        r#"<div><p>The user is <b>{isLoggedIn ? 'currently' : 'not'}</b> logged in.</p><a />{}<div>222 {}</div><span>{} 222</span></div>"#,
        // Output codes after transformed with plugin
        r#"<div><p>The user is <b>{isLoggedIn ? 'currently' : 'not'}</b> logged in.</p><a /><div>222 </div><span> 222</span></div>;"#
    );

    test!(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| as_folder(TransformVisitor),
        remove_empty_expression_with_comments,
        // Input codes
        r#"<div>{/* Hello */}<a /></div>"#,
        // Output codes after transformed with plugin
        r#"<div><a /></div>;"#
    );

   

}
