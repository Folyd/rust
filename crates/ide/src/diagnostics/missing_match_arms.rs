use hir::InFile;

use crate::diagnostics::{Diagnostic, DiagnosticsContext};

// Diagnostic: missing-match-arm
//
// This diagnostic is triggered if `match` block is missing one or more match arms.
pub(super) fn missing_match_arms(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::MissingMatchArms,
) -> Diagnostic {
    Diagnostic::new(
        "missing-match-arm",
        "missing match arm",
        ctx.sema.diagnostics_display_range(InFile::new(d.file, d.match_expr.clone().into())).range,
    )
}

#[cfg(test)]
pub(super) mod tests {
    use crate::diagnostics::tests::check_diagnostics;

    #[test]
    fn empty_tuple() {
        check_diagnostics(
            r#"
fn main() {
    match () { }
        //^^ missing match arm
    match (()) { }
        //^^^^ missing match arm

    match () { _ => (), }
    match () { () => (), }
    match (()) { (()) => (), }
}
"#,
        );
    }

    #[test]
    fn tuple_of_two_empty_tuple() {
        check_diagnostics(
            r#"
fn main() {
    match ((), ()) { }
        //^^^^^^^^ missing match arm

    match ((), ()) { ((), ()) => (), }
}
"#,
        );
    }

    #[test]
    fn boolean() {
        check_diagnostics(
            r#"
fn test_main() {
    match false { }
        //^^^^^ missing match arm
    match false { true => (), }
        //^^^^^ missing match arm
    match (false, true) {}
        //^^^^^^^^^^^^^ missing match arm
    match (false, true) { (true, true) => (), }
        //^^^^^^^^^^^^^ missing match arm
    match (false, true) {
        //^^^^^^^^^^^^^ missing match arm
        (false, true) => (),
        (false, false) => (),
        (true, false) => (),
    }
    match (false, true) { (true, _x) => (), }
        //^^^^^^^^^^^^^ missing match arm

    match false { true => (), false => (), }
    match (false, true) {
        (false, _) => (),
        (true, false) => (),
        (_, true) => (),
    }
    match (false, true) {
        (true, true) => (),
        (true, false) => (),
        (false, true) => (),
        (false, false) => (),
    }
    match (false, true) {
        (true, _x) => (),
        (false, true) => (),
        (false, false) => (),
    }
    match (false, true, false) {
        (false, ..) => (),
        (true, ..) => (),
    }
    match (false, true, false) {
        (.., false) => (),
        (.., true) => (),
    }
    match (false, true, false) { (..) => (), }
}
"#,
        );
    }

    #[test]
    fn tuple_of_tuple_and_bools() {
        check_diagnostics(
            r#"
fn main() {
    match (false, ((), false)) {}
        //^^^^^^^^^^^^^^^^^^^^ missing match arm
    match (false, ((), false)) { (true, ((), true)) => (), }
        //^^^^^^^^^^^^^^^^^^^^ missing match arm
    match (false, ((), false)) { (true, _) => (), }
        //^^^^^^^^^^^^^^^^^^^^ missing match arm

    match (false, ((), false)) {
        (true, ((), true)) => (),
        (true, ((), false)) => (),
        (false, ((), true)) => (),
        (false, ((), false)) => (),
    }
    match (false, ((), false)) {
        (true, ((), true)) => (),
        (true, ((), false)) => (),
        (false, _) => (),
    }
}
"#,
        );
    }

    #[test]
    fn enums() {
        check_diagnostics(
            r#"
enum Either { A, B, }

fn main() {
    match Either::A { }
        //^^^^^^^^^ missing match arm
    match Either::B { Either::A => (), }
        //^^^^^^^^^ missing match arm

    match &Either::B {
        //^^^^^^^^^^ missing match arm
        Either::A => (),
    }

    match Either::B {
        Either::A => (), Either::B => (),
    }
    match &Either::B {
        Either::A => (), Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn enum_containing_bool() {
        check_diagnostics(
            r#"
enum Either { A(bool), B }

fn main() {
    match Either::B { }
        //^^^^^^^^^ missing match arm
    match Either::B {
        //^^^^^^^^^ missing match arm
        Either::A(true) => (), Either::B => ()
    }

    match Either::B {
        Either::A(true) => (),
        Either::A(false) => (),
        Either::B => (),
    }
    match Either::B {
        Either::B => (),
        _ => (),
    }
    match Either::B {
        Either::A(_) => (),
        Either::B => (),
    }

}
        "#,
        );
    }

    #[test]
    fn enum_different_sizes() {
        check_diagnostics(
            r#"
enum Either { A(bool), B(bool, bool) }

fn main() {
    match Either::A(false) {
        //^^^^^^^^^^^^^^^^ missing match arm
        Either::A(_) => (),
        Either::B(false, _) => (),
    }

    match Either::A(false) {
        Either::A(_) => (),
        Either::B(true, _) => (),
        Either::B(false, _) => (),
    }
    match Either::A(false) {
        Either::A(true) | Either::A(false) => (),
        Either::B(true, _) => (),
        Either::B(false, _) => (),
    }
}
"#,
        );
    }

    #[test]
    fn tuple_of_enum_no_diagnostic() {
        check_diagnostics(
            r#"
enum Either { A(bool), B(bool, bool) }
enum Either2 { C, D }

fn main() {
    match (Either::A(false), Either2::C) {
        (Either::A(true), _) | (Either::A(false), _) => (),
        (Either::B(true, _), Either2::C) => (),
        (Either::B(false, _), Either2::C) => (),
        (Either::B(_, _), Either2::D) => (),
    }
}
"#,
        );
    }

    #[test]
    fn or_pattern_no_diagnostic() {
        check_diagnostics(
            r#"
enum Either {A, B}

fn main() {
    match (Either::A, Either::B) {
        (Either::A | Either::B, _) => (),
    }
}"#,
        )
    }

    #[test]
    fn mismatched_types() {
        // Match statements with arms that don't match the
        // expression pattern do not fire this diagnostic.
        check_diagnostics(
            r#"
enum Either { A, B }
enum Either2 { C, D }

fn main() {
    match Either::A {
        Either2::C => (),
    //  ^^^^^^^^^^ Internal: match check bailed out
        Either2::D => (),
    }
    match (true, false) {
        (true, false, true) => (),
    //  ^^^^^^^^^^^^^^^^^^^ Internal: match check bailed out
        (true) => (),
    }
    match (true, false) { (true,) => {} }
    //                    ^^^^^^^ Internal: match check bailed out
    match (0) { () => () }
            //  ^^ Internal: match check bailed out
    match Unresolved::Bar { Unresolved::Baz => () }
}
        "#,
        );
    }

    #[test]
    fn mismatched_types_in_or_patterns() {
        check_diagnostics(
            r#"
fn main() {
    match false { true | () => {} }
    //            ^^^^^^^^^ Internal: match check bailed out
    match (false,) { (true | (),) => {} }
    //               ^^^^^^^^^^^^ Internal: match check bailed out
}
"#,
        );
    }

    #[test]
    fn malformed_match_arm_tuple_enum_missing_pattern() {
        // We are testing to be sure we don't panic here when the match
        // arm `Either::B` is missing its pattern.
        check_diagnostics(
            r#"
enum Either { A, B(u32) }

fn main() {
    match Either::A {
        Either::A => (),
        Either::B() => (),
    }
}
"#,
        );
    }

    #[test]
    fn malformed_match_arm_extra_fields() {
        check_diagnostics(
            r#"
enum A { B(isize, isize), C }
fn main() {
    match A::B(1, 2) {
        A::B(_, _, _) => (),
    //  ^^^^^^^^^^^^^ Internal: match check bailed out
    }
    match A::B(1, 2) {
        A::C(_) => (),
    //  ^^^^^^^ Internal: match check bailed out
    }
}
"#,
        );
    }

    #[test]
    fn expr_diverges() {
        check_diagnostics(
            r#"
enum Either { A, B }

fn main() {
    match loop {} {
        Either::A => (),
    //  ^^^^^^^^^ Internal: match check bailed out
        Either::B => (),
    }
    match loop {} {
        Either::A => (),
    //  ^^^^^^^^^ Internal: match check bailed out
    }
    match loop { break Foo::A } {
        //^^^^^^^^^^^^^^^^^^^^^ missing match arm
        Either::A => (),
    }
    match loop { break Foo::A } {
        Either::A => (),
        Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn expr_partially_diverges() {
        check_diagnostics(
            r#"
enum Either<T> { A(T), B }

fn foo() -> Either<!> { Either::B }
fn main() -> u32 {
    match foo() {
        Either::A(val) => val,
        Either::B => 0,
    }
}
"#,
        );
    }

    #[test]
    fn enum_record() {
        check_diagnostics(
            r#"
enum Either { A { foo: bool }, B }

fn main() {
    let a = Either::A { foo: true };
    match a { }
        //^ missing match arm
    match a { Either::A { foo: true } => () }
        //^ missing match arm
    match a {
        Either::A { } => (),
      //^^^^^^^^^ Missing structure fields:
      //        | - foo
        Either::B => (),
    }
    match a {
        //^ missing match arm
        Either::A { } => (),
    } //^^^^^^^^^ Missing structure fields:
      //        | - foo

    match a {
        Either::A { foo: true } => (),
        Either::A { foo: false } => (),
        Either::B => (),
    }
    match a {
        Either::A { foo: _ } => (),
        Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn enum_record_fields_out_of_order() {
        check_diagnostics(
            r#"
enum Either {
    A { foo: bool, bar: () },
    B,
}

fn main() {
    let a = Either::A { foo: true, bar: () };
    match a {
        //^ missing match arm
        Either::A { bar: (), foo: false } => (),
        Either::A { foo: true, bar: () } => (),
    }

    match a {
        Either::A { bar: (), foo: false } => (),
        Either::A { foo: true, bar: () } => (),
        Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn enum_record_ellipsis() {
        check_diagnostics(
            r#"
enum Either {
    A { foo: bool, bar: bool },
    B,
}

fn main() {
    let a = Either::B;
    match a {
        //^ missing match arm
        Either::A { foo: true, .. } => (),
        Either::B => (),
    }
    match a {
        //^ missing match arm
        Either::A { .. } => (),
    }

    match a {
        Either::A { foo: true, .. } => (),
        Either::A { foo: false, .. } => (),
        Either::B => (),
    }

    match a {
        Either::A { .. } => (),
        Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn enum_tuple_partial_ellipsis() {
        check_diagnostics(
            r#"
enum Either {
    A(bool, bool, bool, bool),
    B,
}

fn main() {
    match Either::B {
        //^^^^^^^^^ missing match arm
        Either::A(true, .., true) => (),
        Either::A(true, .., false) => (),
        Either::A(false, .., false) => (),
        Either::B => (),
    }
    match Either::B {
        //^^^^^^^^^ missing match arm
        Either::A(true, .., true) => (),
        Either::A(true, .., false) => (),
        Either::A(.., true) => (),
        Either::B => (),
    }

    match Either::B {
        Either::A(true, .., true) => (),
        Either::A(true, .., false) => (),
        Either::A(false, .., true) => (),
        Either::A(false, .., false) => (),
        Either::B => (),
    }
    match Either::B {
        Either::A(true, .., true) => (),
        Either::A(true, .., false) => (),
        Either::A(.., true) => (),
        Either::A(.., false) => (),
        Either::B => (),
    }
}
"#,
        );
    }

    #[test]
    fn never() {
        check_diagnostics(
            r#"
enum Never {}

fn enum_(never: Never) {
    match never {}
}
fn enum_ref(never: &Never) {
    match never {}
        //^^^^^ missing match arm
}
fn bang(never: !) {
    match never {}
}
"#,
        );
    }

    #[test]
    fn unknown_type() {
        check_diagnostics(
            r#"
enum Option<T> { Some(T), None }

fn main() {
    // `Never` is deliberately not defined so that it's an uninferred type.
    match Option::<Never>::None {
        None => (),
        Some(never) => match never {},
    //  ^^^^^^^^^^^ Internal: match check bailed out
    }
    match Option::<Never>::None {
        //^^^^^^^^^^^^^^^^^^^^^ missing match arm
        Option::Some(_never) => {},
    }
}
"#,
        );
    }

    #[test]
    fn tuple_of_bools_with_ellipsis_at_end_missing_arm() {
        check_diagnostics(
            r#"
fn main() {
    match (false, true, false) {
        //^^^^^^^^^^^^^^^^^^^^ missing match arm
        (false, ..) => (),
    }
}"#,
        );
    }

    #[test]
    fn tuple_of_bools_with_ellipsis_at_beginning_missing_arm() {
        check_diagnostics(
            r#"
fn main() {
    match (false, true, false) {
        //^^^^^^^^^^^^^^^^^^^^ missing match arm
        (.., false) => (),
    }
}"#,
        );
    }

    #[test]
    fn tuple_of_bools_with_ellipsis_in_middle_missing_arm() {
        check_diagnostics(
            r#"
fn main() {
    match (false, true, false) {
        //^^^^^^^^^^^^^^^^^^^^ missing match arm
        (true, .., false) => (),
    }
}"#,
        );
    }

    #[test]
    fn record_struct() {
        check_diagnostics(
            r#"struct Foo { a: bool }
fn main(f: Foo) {
    match f {}
        //^ missing match arm
    match f { Foo { a: true } => () }
        //^ missing match arm
    match &f { Foo { a: true } => () }
        //^^ missing match arm
    match f { Foo { a: _ } => () }
    match f {
        Foo { a: true } => (),
        Foo { a: false } => (),
    }
    match &f {
        Foo { a: true } => (),
        Foo { a: false } => (),
    }
}
"#,
        );
    }

    #[test]
    fn tuple_struct() {
        check_diagnostics(
            r#"struct Foo(bool);
fn main(f: Foo) {
    match f {}
        //^ missing match arm
    match f { Foo(true) => () }
        //^ missing match arm
    match f {
        Foo(true) => (),
        Foo(false) => (),
    }
}
"#,
        );
    }

    #[test]
    fn unit_struct() {
        check_diagnostics(
            r#"struct Foo;
fn main(f: Foo) {
    match f {}
        //^ missing match arm
    match f { Foo => () }
}
"#,
        );
    }

    #[test]
    fn record_struct_ellipsis() {
        check_diagnostics(
            r#"struct Foo { foo: bool, bar: bool }
fn main(f: Foo) {
    match f { Foo { foo: true, .. } => () }
        //^ missing match arm
    match f {
        //^ missing match arm
        Foo { foo: true, .. } => (),
        Foo { bar: false, .. } => ()
    }
    match f { Foo { .. } => () }
    match f {
        Foo { foo: true, .. } => (),
        Foo { foo: false, .. } => ()
    }
}
"#,
        );
    }

    #[test]
    fn internal_or() {
        check_diagnostics(
            r#"
fn main() {
    enum Either { A(bool), B }
    match Either::B {
        //^^^^^^^^^ missing match arm
        Either::A(true | false) => (),
    }
}
"#,
        );
    }

    #[test]
    fn no_panic_at_unimplemented_subpattern_type() {
        check_diagnostics(
            r#"
struct S { a: char}
fn main(v: S) {
    match v { S{ a }      => {} }
    match v { S{ a: _x }  => {} }
    match v { S{ a: 'a' } => {} }
            //^^^^^^^^^^^ Internal: match check bailed out
    match v { S{..}       => {} }
    match v { _           => {} }
    match v { }
        //^ missing match arm
}
"#,
        );
    }

    #[test]
    fn binding() {
        check_diagnostics(
            r#"
fn main() {
    match true {
        _x @ true => {}
        false     => {}
    }
    match true { _x @ true => {} }
        //^^^^ missing match arm
}
"#,
        );
    }

    #[test]
    fn binding_ref_has_correct_type() {
        // Asserts `PatKind::Binding(ref _x): bool`, not &bool.
        // If that's not true match checking will panic with "incompatible constructors"
        // FIXME: make facilities to test this directly like `tests::check_infer(..)`
        check_diagnostics(
            r#"
enum Foo { A }
fn main() {
    // FIXME: this should not bail out but current behavior is such as the old algorithm.
    // ExprValidator::validate_match(..) checks types of top level patterns incorrecly.
    match Foo::A {
        ref _x => {}
    //  ^^^^^^ Internal: match check bailed out
        Foo::A => {}
    }
    match (true,) {
        (ref _x,) => {}
        (true,) => {}
    }
}
"#,
        );
    }

    #[test]
    fn enum_non_exhaustive() {
        check_diagnostics(
            r#"
//- /lib.rs crate:lib
#[non_exhaustive]
pub enum E { A, B }
fn _local() {
    match E::A { _ => {} }
    match E::A {
        E::A => {}
        E::B => {}
    }
    match E::A {
        E::A | E::B => {}
    }
}

//- /main.rs crate:main deps:lib
use lib::E;
fn main() {
    match E::A { _ => {} }
    match E::A {
        //^^^^ missing match arm
        E::A => {}
        E::B => {}
    }
    match E::A {
        //^^^^ missing match arm
        E::A | E::B => {}
    }
}
"#,
        );
    }

    #[test]
    fn match_guard() {
        check_diagnostics(
            r#"
fn main() {
    match true {
        true if false => {}
        true          => {}
        false         => {}
    }
    match true {
        //^^^^ missing match arm
        true if false => {}
        false         => {}
    }
}
"#,
        );
    }

    #[test]
    fn pattern_type_is_of_substitution() {
        cov_mark::check!(match_check_wildcard_expanded_to_substitutions);
        check_diagnostics(
            r#"
struct Foo<T>(T);
struct Bar;
fn main() {
    match Foo(Bar) {
        _ | Foo(Bar) => {}
    }
}
"#,
        );
    }

    #[test]
    fn record_struct_no_such_field() {
        check_diagnostics(
            r#"
struct Foo { }
fn main(f: Foo) {
    match f { Foo { bar } => () }
    //        ^^^^^^^^^^^ Internal: match check bailed out
}
"#,
        );
    }

    #[test]
    fn match_ergonomics_issue_9095() {
        check_diagnostics(
            r#"
enum Foo<T> { A(T) }
fn main() {
    match &Foo::A(true) {
        _ => {}
        Foo::A(_) => {}
    }
}
"#,
        );
    }

    mod false_negatives {
        //! The implementation of match checking here is a work in progress. As we roll this out, we
        //! prefer false negatives to false positives (ideally there would be no false positives). This
        //! test module should document known false negatives. Eventually we will have a complete
        //! implementation of match checking and this module will be empty.
        //!
        //! The reasons for documenting known false negatives:
        //!
        //!   1. It acts as a backlog of work that can be done to improve the behavior of the system.
        //!   2. It ensures the code doesn't panic when handling these cases.
        use super::*;

        #[test]
        fn integers() {
            // We don't currently check integer exhaustiveness.
            check_diagnostics(
                r#"
fn main() {
    match 5 {
        10 => (),
    //  ^^ Internal: match check bailed out
        11..20 => (),
    }
}
"#,
            );
        }

        #[test]
        fn reference_patterns_at_top_level() {
            check_diagnostics(
                r#"
fn main() {
    match &false {
        &true => {}
    //  ^^^^^ Internal: match check bailed out
    }
}
            "#,
            );
        }

        #[test]
        fn reference_patterns_in_fields() {
            check_diagnostics(
                r#"
fn main() {
    match (&false,) {
        (true,) => {}
    //  ^^^^^^^ Internal: match check bailed out
    }
    match (&false,) {
        (&true,) => {}
    //  ^^^^^^^^ Internal: match check bailed out
    }
}
            "#,
            );
        }
    }
}
