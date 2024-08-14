use oxc_ast::ast::{BindingPatternKind, VariableDeclarationKind, VariableDeclarator};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use std::fmt::Pointer;
use oxc_ast::syntax_directed_operations::BoundNames;
use oxc_index::IdxSliceIndex;
use oxc_syntax::scope::ScopeId;
use crate::{
    context::LintContext
    ,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferConst(Box<PreferConstOptions>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum Destructuring {
    All,
    #[default]
    Any,
}


#[derive(Debug, Clone)]
pub struct PreferConstOptions {
    destructuring: Destructuring,
    ignore_read_before_assign: bool,
}

impl Default for PreferConstOptions {
    fn default() -> Self {
        // we follow the eslint defaults
        Self {
            destructuring: Destructuring::Any,
            ignore_read_before_assign: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// It ensures that if a variable is never reassigned, the const keyword is used instead of let or var.
    ///
    /// ### Why is this bad?
    ///
    /// If a variable is never reassigned, using the const declaration is better because it makes
    /// it clear that the variable can't be reassigned.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let a = 3;
    /// console.log(a);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = 3;
    /// console.log(a);
    /// ```
    PreferConst,
    correctness, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferConst {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut options = PreferConstOptions::default();
        if let Some(obj) = value.as_object() {
            if let Some(destructuring) = obj.get("destructuring") {
                options.destructuring = match destructuring.as_str() {
                    Some("all") => Destructuring::All,
                    _ => Destructuring::Any,
                }
            }
            if let Some(ignore_read_before_assign) = obj.get("ignoreReadBeforeAssign") {
                options.ignore_read_before_assign = ignore_read_before_assign.as_bool().unwrap_or(false)
            }
        }
        Self(Box::new(options))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(dec) = node.kind() {
            if dec.kind != VariableDeclarationKind::Const {
                let init_declaration = &dec.declarations[0];
                let (var_name, symbol_id) = match &init_declaration.id.kind {
                    BindingPatternKind::BindingIdentifier(id) => (&id.name, id.symbol_id.get()),
                    _ => return,
                };
                let references = ctx.semantic().symbol_references(symbol_id.unwrap());

                let filtered_declarations: Vec<&VariableDeclarator> = dec.declarations.iter().filter(|declaration| {
                    if let Some(init) = &declaration.init {
                        return false;
                    };
                    return true;
                }).collect();
                if filtered_declarations.len() > 1 {
                    ctx.diagnostic(
                        OxcDiagnostic::warn("Variable is never reassigned, use const instead.")
                            .with_label(Span::new(dec.span.start, dec.span.start + 3)),
                    )
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ("var x = 0;", None),
        // ("let x;", None),
        // ("let x; { x = 0; } foo(x);", None),
        // ("let x = 0; x = 1;", None),
        // ("const x = 0;", None),
        // ("for (let i = 0, end = 10; i < end; ++i) {}", None),
        // ("for (let i in [1,2,3]) { i = 0; }", None),
        // ("for (let x of [1,2,3]) { x = 0; }", None),
        // ("(function() { var x = 0; })();", None),
        // ("(function() { let x; })();", None),
        // ("(function() { let x; { x = 0; } foo(x); })();", None),
        // ("(function() { let x = 0; x = 1; })();", None),
        // ("(function() { const x = 0; })();", None),
        // ("(function() { for (let i = 0, end = 10; i < end; ++i) {} })();", None),
        // ("(function() { for (let i in [1,2,3]) { i = 0; } })();", None),
        // ("(function() { for (let x of [1,2,3]) { x = 0; } })();", None),
        // ("(function(x = 0) { })();", None),
        // ("let a; while (a = foo());", None),
        // ("let a; do {} while (a = foo());", None),
        // ("let a; for (; a = foo(); );", None),
        // ("let a; for (;; ++a);", None),
        // ("let a; for (const {b = ++a} in foo());", None),
        // ("let a; for (const {b = ++a} of foo());", None),
        // ("let a; for (const x of [1,2,3]) { if (a) {} a = foo(); }", None),
        // ("let a; for (const x of [1,2,3]) { a = a || foo(); bar(a); }", None),
        // ("let a; for (const x of [1,2,3]) { foo(++a); }", None),
        // ("let a; function foo() { if (a) {} a = bar(); }", None),
        // ("let a; function foo() { a = a || bar(); baz(a); }", None),
        // ("let a; function foo() { bar(++a); }", None),
        // (
        //     "let id;
		// 	function foo() {
		// 	    if (typeof id !== 'undefined') {
		// 	        return;
		// 	    }
		// 	    id = setInterval(() => {}, 250);
		// 	}
		// 	foo();
		// 	",
        //     None,
        // ),
        // ("/*exported a*/ let a; function init() { a = foo(); }", None),
        // ("/*exported a*/ let a = 1", None),
        // ("let a; if (true) a = 0; foo(a);", None),
        // (
        //     "
		// 	        (function (a) {
		// 	            let b;
		// 	            ({ a, b } = obj);
		// 	        })();
		// 	        ",
        //     None,
        // ),
        // (
        //     "
		// 	        (function (a) {
		// 	            let b;
		// 	            ([ a, b ] = obj);
		// 	        })();
		// 	        ",
        //     None,
        // ),
        // ("var a; { var b; ({ a, b } = obj); }", None),
        // ("let a; { let b; ({ a, b } = obj); }", None),
        // ("var a; { var b; ([ a, b ] = obj); }", None),
        // ("let a; { let b; ([ a, b ] = obj); }", None),
        // ("let x; { x = 0; foo(x); }", None),
        // ("(function() { let x; { x = 0; foo(x); } })();", None),
        // ("let x; for (const a of [1,2,3]) { x = foo(); bar(x); }", None),
        // ("(function() { let x; for (const a of [1,2,3]) { x = foo(); bar(x); } })();", None),
        // ("let x; for (x of array) { x; }", None),
        // ("let {a, b} = obj; b = 0;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        ("let a, b; ({a, b} = obj); b++;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ), // {                "parser": require(fixtureParser("babel-eslint5/destructuring-object-spread"))            },
        ("let predicate; [typeNode.returnType, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType, ...predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType,, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType=5, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [[typeNode.returnType=5], predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [[typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType, [predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, [typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:typeNode.returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:typeNode.returnType, ...predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let a; const b = {}; ({ a, c: b.c } = func());", None), // { "ecmaVersion": 2018 },
        (
            "let x; function foo() { bar(x); } x = 0;",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ),
        ("const x = [1,2]; let y; [,y] = x; y = 0;", None),
        ("const x = [1,2,3]; let y, z; [y,,z] = x; y = 0; z = 0;", None),
        ("class C { static { let a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("let a; class C { static { a = 1; } }", None),     // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) { a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) a = 1; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) { ({ a, b } = foo); } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        (
            "class C { static { a; } } let a = 1; ",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { () => a; let a = 1; } };",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("let x = 1; foo(x);", None),
        ("for (let i in [1,2,3]) { foo(i); }", None),
        ("for (let x of [1,2,3]) { foo(x); }", None),
        ("let [x = -1, y] = [1,2]; y = 0;", None),
        ("let {a: x = -1, b: y} = {a:1,b:2}; y = 0;", None),
        ("(function() { let x = 1; foo(x); })();", None),
        ("(function() { for (let i in [1,2,3]) { foo(i); } })();", None),
        ("(function() { for (let x of [1,2,3]) { foo(x); } })();", None),
        ("(function() { let [x = -1, y] = [1,2]; y = 0; })();", None),
        ("let f = (function() { let g = x; })(); f = 1;", None),
        ("(function() { let {a: x = -1, b: y} = {a:1,b:2}; y = 0; })();", None),
        ("let x = 0; { let x = 1; foo(x); } x = 0;", None),
        ("for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }", None),
        ("for (let i in [1,2,3]) { let x = 1; foo(x); }", None),
        (
            "var foo = function() {
			    for (const b of c) {
			       let a;
			       a = 1;
			   }
			};",
            None,
        ),
        (
            "var foo = function() {
			    for (const b of c) {
			       let a;
			       ({a} = 1);
			   }
			};",
            None,
        ),
        ("let x; x = 0;", None),
        ("switch (a) { case 0: let x; x = 0; }", None),
        ("(function() { let x; x = 1; })();", None),
        (
            "let {a = 0, b} = obj; b = 0; foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a: {b, c}} = {a: {b: 1, c: 2}}; b = 3;",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a: {b, c}} = {a: {b: 1, c: 2}}",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let a, b; ({a = 0, b} = obj); b = 0; foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        ("let {a = 0, b} = obj; foo(a, b);", Some(serde_json::json!([{ "destructuring": "all" }]))),
        ("let [a] = [1]", Some(serde_json::json!([]))),
        ("let {a} = obj", Some(serde_json::json!([]))),
        (
            "let a, b; ({a = 0, b} = obj); foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // {                "parser": require(fixtureParser("babel-eslint5/destructuring-object-spread"))            },
        ("let x; function foo() { bar(x); } x = 0;", None),
        ("/*eslint custom/use-x:error*/ let x = 1", None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } },
        ("/*eslint custom/use-x:error*/ { let x = 1 }", None),
        ("let { foo, bar } = baz;", None),
        ("const x = [1,2]; let [,y] = x;", None),
        ("const x = [1,2,3]; let [y,,z] = x;", None),
        ("let predicate; [, {foo:returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:returnType, predicate}, ...bar ] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:returnType, ...predicate} ] = foo();", None), // { "ecmaVersion": 2018 },
        ("let x = 'x', y = 'y';", None),
        ("let x = 'x', y = 'y'; x = 1", None),
        ("let x = 1, y = 'y'; let z = 1;", None),
        ("let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;", None),
        ("let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }", None),
        ("let someFunc = () => { let a = 1, b = 2; foo(a, b) }", None),
        ("let {a, b} = c, d;", None),
        ("let {a, b, c} = {}, e, f;", None),
        (
            "function a() {
			let foo = 0,
			  bar = 1;
			foo = 1;
			}
			function b() {
			let foo = 0,
			  bar = 2;
			foo = 2;
			}",
            None,
        ),
        ("/*eslint no-undef-init:error*/ let foo = undefined;", None),
        ("let a = 1; class C { static { a; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { a; } } let a = 1;", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; } }", None),    // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { let a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; if (foo) { a; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { let a; a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 1; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let { a, b } = foo; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; let b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 0; console.log(a); } }", None), // { "ecmaVersion": 2022 },
        (
            "
			            let { itemId, list } = {},
			            obj = [],
			            total = 0;
			            total = 9;
			            console.log(itemId, list, obj, total);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            let { itemId, list } = {},
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            let [ itemId, list ] = [],
			            total = 0;
			            total = 9;
			            console.log(itemId, list, total);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
			            let [ itemId, list ] = [],
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fix = vec![
        ("let x = 1; foo(x);", "const x = 1; foo(x);", None),
        ("for (let i in [1,2,3]) { foo(i); }", "for (const i in [1,2,3]) { foo(i); }", None),
        ("for (let x of [1,2,3]) { foo(x); }", "for (const x of [1,2,3]) { foo(x); }", None),
        (
            "(function() { let x = 1; foo(x); })();",
            "(function() { const x = 1; foo(x); })();",
            None,
        ),
        (
            "(function() { for (let i in [1,2,3]) { foo(i); } })();",
            "(function() { for (const i in [1,2,3]) { foo(i); } })();",
            None,
        ),
        (
            "(function() { for (let x of [1,2,3]) { foo(x); } })();",
            "(function() { for (const x of [1,2,3]) { foo(x); } })();",
            None,
        ),
        (
            "let f = (function() { let g = x; })(); f = 1;",
            "let f = (function() { const g = x; })(); f = 1;",
            None,
        ),
        (
            "let x = 0; { let x = 1; foo(x); } x = 0;",
            "let x = 0; { const x = 1; foo(x); } x = 0;",
            None,
        ),
        (
            "for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }",
            "for (let i = 0; i < 10; ++i) { const x = 1; foo(x); }",
            None,
        ),
        (
            "for (let i in [1,2,3]) { let x = 1; foo(x); }",
            "for (const i in [1,2,3]) { const x = 1; foo(x); }",
            None,
        ),
        (
            "let {a: {b, c}} = {a: {b: 1, c: 2}}",
            "const {a: {b, c}} = {a: {b: 1, c: 2}}",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let {a = 0, b} = obj; foo(a, b);",
            "const {a = 0, b} = obj; foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        ("let [a] = [1]", "const [a] = [1]", Some(serde_json::json!([]))),
        ("let {a} = obj", "const {a} = obj", Some(serde_json::json!([]))),
        (
            "/*eslint custom/use-x:error*/ let x = 1",
            "/*eslint custom/use-x:error*/ const x = 1",
            None,
        ),
        (
            "/*eslint custom/use-x:error*/ { let x = 1 }",
            "/*eslint custom/use-x:error*/ { const x = 1 }",
            None,
        ),
        ("let { foo, bar } = baz;", "const { foo, bar } = baz;", None),
        ("const x = [1,2]; let [,y] = x;", "const x = [1,2]; const [,y] = x;", None),
        ("const x = [1,2,3]; let [y,,z] = x;", "const x = [1,2,3]; const [y,,z] = x;", None),
        ("let x = 'x', y = 'y';", "const x = 'x', y = 'y';", None),
        ("let x = 1, y = 'y'; let z = 1;", "const x = 1, y = 'y'; const z = 1;", None),
        (
            "let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
            "const { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
            None,
        ),
        (
            "let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }",
            "const x = 'x', y = 'y'; function someFunc() { const a = 1, b = 2; foo(a, b) }",
            None,
        ),
        (
            "let someFunc = () => { let a = 1, b = 2; foo(a, b) }",
            "const someFunc = () => { let a = 1, b = 2; foo(a, b) }",
            None,
        ),
        (
            "/*eslint no-undef-init:error*/ let foo = undefined;",
            "/*eslint no-undef-init:error*/ const foo = undefined;",
            None,
        ),
        ("let a = 1; class C { static { a; } }", "const a = 1; class C { static { a; } }", None),
        ("class C { static { a; } } let a = 1;", "class C { static { a; } } const a = 1;", None),
        ("class C { static { let a = 1; } }", "class C { static { const a = 1; } }", None),
        (
            "class C { static { if (foo) { let a = 1; } } }",
            "class C { static { if (foo) { const a = 1; } } }",
            None,
        ),
        (
            "class C { static { let a = 1; if (foo) { a; } } }",
            "class C { static { const a = 1; if (foo) { a; } } }",
            None,
        ),
        (
            "class C { static { let { a, b } = foo; } }",
            "class C { static { const { a, b } = foo; } }",
            None,
        ),
        (
            "
			            let { itemId, list } = {},
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            "
			            const { itemId, list } = {},
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ),
        (
            "
			            let [ itemId, list ] = [],
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            "
			            const [ itemId, list ] = [],
			            obj = [];
			            console.log(itemId, list, obj);
			            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ),
    ];
    Tester::new(PreferConst::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
