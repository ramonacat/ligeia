pub mod ast;

lalrpop_util::lalrpop_mod!(
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        clippy::missing_const_for_fn,
        clippy::unnecessary_wraps,
        clippy::redundant_pub_crate,
        clippy::unicode_not_nfc,
        clippy::cloned_instead_of_copied,
        clippy::elidable_lifetime_names,
        clippy::no_effect_underscore_binding,
        clippy::too_many_lines,
        clippy::cast_sign_loss,
        clippy::option_if_let_else,
        clippy::use_self,
        clippy::unnested_or_patterns,
        clippy::match_same_arms
    )]
    grammar,
    "/parser/grammar.rs"
);

/// # Panics
/// Will panic in case the file fails to parse
///
/// TODO return readable parse errors to the user
pub fn parse(filename: &str, code: &str) -> ast::SourceFile {
    grammar::SourceFileParser::new()
        .parse(filename, code)
        .unwrap()
}
