use ligeia_compiler_lib::{
    analysis::{self, AnalysisError, type_check::TypeCheckError},
    parser::{self, ast},
};

#[test]
fn mismatched_return_type() {
    let file = parser::parse("main", "fn test() { return 123; }");
    let result = analysis::analyse(&[file]).unwrap_err();

    assert_eq!(
        AnalysisError::TypeCheck(TypeCheckError::MismatchedReturnType(
            ast::Type::Unit,
            ast::Type::U64
        )),
        result
    );
}
