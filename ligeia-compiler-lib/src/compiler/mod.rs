use std::collections::HashMap;

use eisheth::{
    function::{declaration::FunctionSignature, instruction_builder::InstructionBuilder},
    module::DeclaredFunctionDescriptor,
    package::{Package, builder::PackageBuilder},
    types::{self, OpaqueType, RepresentedAs},
    value::{ConstOrDynamicValue, ConstValue},
};

use crate::parser::ast::{self, Expression, FunctionBody, Identifier, SourceFile, Statement};

#[must_use]
pub struct CompiledProgram {
    package: Package,
    main: DeclaredFunctionDescriptor,
}

impl CompiledProgram {
    pub fn into_package(self) -> Package {
        self.package
    }

    pub const fn main(&self) -> DeclaredFunctionDescriptor {
        self.main
    }
}

/// # Panics
/// Will panic if the program fails to compile. This means a bug in the compiler, as all not
/// well-formed programs should be declined at the analysis stage.
pub fn compile(files: Vec<SourceFile>) -> CompiledProgram {
    let mut package_builder = PackageBuilder::new();

    let mut main = None;

    for file in files {
        if let Some(found_main) = compile_file(file, &mut package_builder) {
            main = Some(found_main);
        }
    }

    let build_result = package_builder.build().unwrap();

    eprintln!("{:?}", build_result.messages());

    CompiledProgram {
        package: build_result.into_package(),
        main: main.unwrap(),
    }
}

fn compile_file(
    file: SourceFile,
    package_builder: &mut PackageBuilder,
) -> Option<DeclaredFunctionDescriptor> {
    let module = package_builder.add_module(file.name).unwrap();

    let mut main = None;

    for declaration in file.declarations {
        match declaration {
            ast::Declaration::Function(function) => {
                let is_main = function.name.0 == "main";
                // TODO get the Visibility from source
                let function_id = module.define_function(
                    &FunctionSignature::new(
                        function.name.0.clone(),
                        make_function_type(function.return_type, &function.arguments),
                        function.visibility.into(),
                    ),
                    |f| {
                        compile_function_body(function, f);
                    },
                );

                if is_main {
                    main = Some(function_id);
                }
            }
        }
    }

    main
}

fn compile_function_body(
    function: ast::Function,
    f: &eisheth::function::builder::FunctionBuilder<'_>,
) {
    let mut locals = HashMap::new();

    for (index, argument) in function.arguments.iter().enumerate() {
        locals.insert(argument.name.clone(), f.get_argument(index).unwrap().into());
    }

    match function.body {
        FunctionBody::Extern(_) => todo!(),
        FunctionBody::Statements(statements) => {
            let block = f.create_block("entry");

            for statement in statements {
                match statement {
                    Statement::Expression(_) => todo!(),
                    Statement::Return(expression) => {
                        let result = compile_expression(expression);
                        block.build(|mut i| {
                            // TOOO: verify that the type matches the function's return type
                            // TODO: verify that all code paths return a value
                            let result = result(&mut i, &mut locals);

                            i.r#return(result)
                        });
                    }
                }
            }
        }
    }
}

type ExpressionBuilder = dyn Fn(
    &mut InstructionBuilder,
    // TODO: make this a struct Locals, keep types along with the values
    // TODO: the values should have variants for read-only (like function arguments) and read-write, first being the direct
    // value, second being a pointer to the backing storage
    &mut HashMap<Identifier, ConstOrDynamicValue>,
) -> ConstOrDynamicValue;

fn compile_expression(expression: Expression) -> Box<ExpressionBuilder> {
    match expression {
        Expression::Literal(literal) => match literal {
            ast::Literal::UnsignedInteger(value) => {
                let value: ConstValue = value.into();

                Box::new(move |_, _| value.into())
            }
        },
        Expression::VariableReference(identifier) => {
            Box::new(move |_, locals| *locals.get(&identifier).unwrap())
        }
        Expression::Sum(left, right) => {
            let left = compile_expression(*left);
            let right = compile_expression(*right);

            Box::new(move |i: &mut InstructionBuilder, locals| {
                let left = left(i, locals);
                let right = right(i, locals);

                i.add(&left, &right, "sum")
            })
        }
    }
}

fn make_function_type(return_type: ast::Type, arguments: &[ast::Argument]) -> types::Function {
    types::Function::new(
        make_type(return_type),
        &arguments
            .iter()
            .map(|t| make_type(t.r#type))
            .collect::<Vec<_>>(),
    )
}

fn make_type(return_type: ast::Type) -> OpaqueType {
    match return_type {
        ast::Type::Unit => <()>::representation().into(),
        ast::Type::U64 => u64::representation().into(),
    }
}
