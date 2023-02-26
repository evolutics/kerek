use anyhow::Context;
use nom::branch;
use nom::bytes::complete as bytes;
use nom::character::complete as character;
use nom::combinator;
use nom::multi;
use nom::sequence;
use nom::Parser;
use std::borrow;
use std::collections;
use std::env;
use std::str;

pub fn go<'a>(
    input: &'a str,
    extra_variables: &'a collections::HashMap<String, Option<String>>,
) -> anyhow::Result<borrow::Cow<'a, str>> {
    let (_, expression) = parse_top_level(input).map_err(|error| error.to_owned())?;
    evaluate(expression, extra_variables)
}

enum Expression<'a> {
    Expressions(Vec<Expression<'a>>),
    Literal(&'a str),
    Variable {
        argument: Option<Box<Expression<'a>>>,
        identifier: &'a str,
        requirement: VariableRequirement,
    },
}

enum VariableRequirement {
    OptionalNotEmpty,
    OptionalSet,
    Recommended,
    RequiredNotEmpty,
    RequiredSet,
}

fn parse_top_level(input: &str) -> nom::IResult<&str, Expression> {
    let (input, expressions) =
        multi::many0(branch::alt((parse_expression, parse_any_1_character)))(input)?;
    Ok((input, Expression::Expressions(expressions)))
}

fn parse_expression(input: &str) -> nom::IResult<&str, Expression> {
    let (input, expressions) = multi::many1(branch::alt((
        parse_dollar_sign,
        parse_literal,
        parse_variable,
    )))(input)?;
    Ok((input, Expression::Expressions(expressions)))
}

fn parse_dollar_sign(input: &str) -> nom::IResult<&str, Expression> {
    let (input, _) = bytes::tag("$$")(input)?;
    Ok((input, Expression::Literal("$")))
}

fn parse_literal(input: &str) -> nom::IResult<&str, Expression> {
    let (input, literal) = combinator::recognize(multi::many1(bytes::is_not("$}")))(input)?;
    Ok((input, Expression::Literal(literal)))
}

fn parse_variable(input: &str) -> nom::IResult<&str, Expression> {
    branch::alt((parse_variable_only, parse_variable_with_extras))(input)
}

fn parse_variable_only(input: &str) -> nom::IResult<&str, Expression> {
    let (input, _) = bytes::tag("$")(input)?;
    let (input, identifier) = branch::alt((
        parse_identifier,
        sequence::delimited(bytes::tag("{"), parse_identifier, bytes::tag("}")),
    ))(input)?;

    Ok((
        input,
        Expression::Variable {
            argument: None,
            identifier,
            requirement: VariableRequirement::Recommended,
        },
    ))
}

fn parse_identifier(input: &str) -> nom::IResult<&str, &str> {
    combinator::recognize(sequence::pair(
        branch::alt((character::alpha1, bytes::tag("_"))),
        multi::many0(branch::alt((character::alphanumeric1, bytes::tag("_")))),
    ))(input)
}

fn parse_variable_with_extras(input: &str) -> nom::IResult<&str, Expression> {
    let (input, _) = bytes::tag("${")(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, requirement) = branch::alt((
        bytes::tag("-").map(|_| VariableRequirement::OptionalSet),
        bytes::tag(":-").map(|_| VariableRequirement::OptionalNotEmpty),
        bytes::tag(":?").map(|_| VariableRequirement::RequiredNotEmpty),
        bytes::tag("?").map(|_| VariableRequirement::RequiredSet),
    ))(input)?;
    let (input, argument) = combinator::opt(parse_expression)(input)?;
    let (input, _) = bytes::tag("}")(input)?;

    Ok((
        input,
        Expression::Variable {
            argument: argument.map(Box::new),
            identifier,
            requirement,
        },
    ))
}

fn parse_any_1_character(input: &str) -> nom::IResult<&str, Expression> {
    let (input, character) = combinator::recognize(character::anychar)(input)?;
    Ok((input, Expression::Literal(character)))
}

fn evaluate<'a>(
    expression: Expression<'a>,
    extra_variables: &'a collections::HashMap<String, Option<String>>,
) -> anyhow::Result<borrow::Cow<'a, str>> {
    match expression {
        Expression::Expressions(expressions) => {
            let values = expressions
                .into_iter()
                .map(|expression| evaluate(expression, extra_variables))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(values.concat().into())
        }

        Expression::Literal(literal) => Ok(literal.into()),

        Expression::Variable {
            argument,
            identifier,
            requirement,
        } => {
            let value = look_up_variable(identifier, extra_variables)?;
            let value = match (&requirement, value) {
                (VariableRequirement::OptionalNotEmpty, Some(value))
                | (VariableRequirement::RequiredNotEmpty, Some(value))
                    if value.is_empty() =>
                {
                    None
                }
                (_, value) => value,
            };

            match value {
                None => {
                    let argument = match argument {
                        None => "".into(),
                        Some(argument) => evaluate(*argument, extra_variables)?,
                    };

                    match requirement {
                        VariableRequirement::OptionalNotEmpty
                        | VariableRequirement::OptionalSet => Ok(argument),

                        VariableRequirement::Recommended => {
                            eprintln!(
                                "Warning: missing variable {identifier:?}, \
                                substituting it with empty string."
                            );
                            Ok(argument)
                        }

                        VariableRequirement::RequiredNotEmpty
                        | VariableRequirement::RequiredSet => {
                            let error = Err(anyhow::anyhow!("Missing variable {identifier:?}"));
                            let context = argument.into_owned();
                            if context.is_empty() {
                                error
                            } else {
                                error.context(context)
                            }
                        }
                    }
                }

                Some(value) => Ok(value),
            }
        }
    }
}

fn look_up_variable<'a>(
    identifier: &str,
    extra_variables: &'a collections::HashMap<String, Option<String>>,
) -> anyhow::Result<Option<borrow::Cow<'a, str>>> {
    match extra_variables.get(identifier) {
        None => match env::var(identifier) {
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(value)) => Err(anyhow::anyhow!(
                "Variable {identifier:?} has non-Unicode value {value:?}"
            )),
            Ok(value) => Ok(Some(value.into())),
        },
        Some(value) => Ok(value.as_ref().map(|value| value.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case("$SOME", Some("X"); "0a")]
    #[test_case::test_case("$THING", Some("YZ"); "0b")]
    #[test_case::test_case("${SOME}", Some("X"); "1a")]
    #[test_case::test_case("${THING}", Some("YZ"); "1b")]
    #[test_case::test_case("${SOME:-default}", Some("X"); "2a")]
    #[test_case::test_case("${UNSET:-default}", Some("default"); "2b")]
    #[test_case::test_case("${EMPTY:-default}", Some("default"); "2c")]
    #[test_case::test_case("${SOME-default}", Some("X"); "2d")]
    #[test_case::test_case("${UNSET-default}", Some("default"); "2e")]
    #[test_case::test_case("${EMPTY-default}", Some(""); "2f")]
    #[test_case::test_case("${UNSET-}", Some(""); "2g")]
    #[test_case::test_case("${SOME:?error}", Some("X"); "3a")]
    #[test_case::test_case("${UNSET:?error}", None; "3b")]
    #[test_case::test_case("${EMPTY:?error}", None; "3c")]
    #[test_case::test_case("${SOME?error}", Some("X"); "3d")]
    #[test_case::test_case("${UNSET?error}", None; "3e")]
    #[test_case::test_case("${EMPTY?error}", Some(""); "3f")]
    #[test_case::test_case("${UNSET?}", None; "3g")]
    #[test_case::test_case("${SOME:-${THING}}", Some("X"); "4a")]
    #[test_case::test_case("${UNSET:-${THING}}", Some("YZ"); "4b")]
    #[test_case::test_case("${SOME?$THING}", Some("X"); "4c")]
    #[test_case::test_case("${UNSET?$THING}", None; "4d")]
    #[test_case::test_case("${SOME:-${UNSET:-default}}", Some("X"); "4e")]
    #[test_case::test_case("${UNSET:-${SOME:-default}}", Some("X"); "4f")]
    #[test_case::test_case("${UNSET:-${EMPTY:-default}}", Some("default"); "4g")]
    #[test_case::test_case("$$SOME", Some("$SOME"); "5")]
    #[test_case::test_case("${UNSET}", Some(""); "6")]
    #[test_case::test_case("} { ${SOME} } {", Some("} { X } {"); "7")]
    #[test_case::test_case("0. $$.
1. $SOME.
2. ${SOME}.
3. ${UNSET:-default 0}.
4. ${UNSET-default 1}.
5. ${SOME:?error 0}.
6. ${SOME?error 1}.
7. ${UNSET:-${SOME:-default}}.
8. $$SOME.
9. ${UNSET}.
---", Some("0. $.
1. X.
2. X.
3. default 0.
4. default 1.
5. X.
6. X.
7. X.
8. $SOME.
9. .
---"); "8")]
    fn handles_substitution(input: &str, expected: Option<&str>) -> anyhow::Result<()> {
        assert_eq!(
            go(
                input,
                &[
                    ("EMPTY".into(), Some("".into())),
                    ("SOME".into(), Some("X".into())),
                    ("THING".into(), Some("YZ".into())),
                    ("UNSET".into(), None),
                ]
                .into(),
            )
            .ok(),
            expected.map(|expected| expected.into()),
        );
        Ok(())
    }

    #[test_case::test_case(""; "0")]
    #[test_case::test_case("x"; "1a")]
    #[test_case::test_case("foo"; "1b")]
    #[test_case::test_case("$"; "2a")]
    #[test_case::test_case("-"; "2b")]
    #[test_case::test_case(":"; "2c")]
    #[test_case::test_case("?"; "2d")]
    #[test_case::test_case("{"; "2e")]
    #[test_case::test_case("}"; "2f")]
    #[test_case::test_case("$123_VARIABLE"; "3a")]
    #[test_case::test_case("${123_VARIABLE}"; "3b")]
    #[test_case::test_case("${VARIABLE"; "4a")]
    #[test_case::test_case("${VARIABLE }"; "4b")]
    #[test_case::test_case("${ VARIABLE}"; "4c")]
    #[test_case::test_case("${VARIABLE/foo/bar}"; "5")]
    fn handles_unchanged(input: &str) -> anyhow::Result<()> {
        assert_eq!(go(input, &[].into())?, input);
        Ok(())
    }
}
