use crate::intermediate::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{pair, preceded},
    IResult,
};

use super::{
    asn1_type, asn1_value,
    common::{in_braces, opt_parentheses, skip_ws_and_comments, value_identifier},
    constraint::constraint,
};

/// Tries to parse an ASN1 SEQUENCE OF Value
///
/// *`input` - string slice to be matched against
///
/// `sequence_of_value` will try to match an SEQUENCE OF value declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `SequenceOf` value representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn sequence_or_set_of_value<'a>(input: &'a str) -> IResult<&'a str, ASN1Value> {
    map(
        in_braces(separated_list0(
            skip_ws_and_comments(char(COMMA)),
            skip_ws_and_comments(alt((
                preceded(value_identifier, skip_ws_and_comments(asn1_value)),
                asn1_value,
            ))),
        )),
        |seq| ASN1Value::SequenceOrSetOf(seq),
    )(input)
}

/// Tries to parse an ASN1 SEQUENCE OF
///
/// *`input` - string slice to be matched against
///
/// `sequence_of` will try to match an SEQUENCE OF declaration in the `input` string.
/// If the match succeeds, the parser will consume the match and return the remaining string
/// and a wrapped `SequenceOf` type representing the ASN1 declaration.
/// If the match fails, the parser will not consume the input and will return an error.
pub fn sequence_of<'a>(input: &'a str) -> IResult<&'a str, ASN1Type> {
    map(
        pair(
            preceded(
                skip_ws_and_comments(tag(SEQUENCE)),
                opt(opt_parentheses(constraint)),
            ),
            preceded(
                skip_ws_and_comments(pair(tag(OF), opt(skip_ws_and_comments(value_identifier)))),
                asn1_type,
            ),
        ),
        |m| ASN1Type::SequenceOf(m.into()),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::intermediate::{
        constraints::*,
        information_object::{ObjectSet, ObjectSetValue},
        types::*,
        *,
    };

    use crate::parser::sequence_of;

    #[test]
    fn parses_simple_sequence_of() {
        assert_eq!(
            sequence_of("SEQUENCE OF BOOLEAN").unwrap().1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![],
                r#type: Box::new(ASN1Type::Boolean(Boolean {
                    constraints: vec![]
                }))
            })
        );
    }

    #[test]
    fn parses_simple_sequence_of_elsewhere_declared_type() {
        assert_eq!(
            sequence_of("SEQUENCE OF Things").unwrap().1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![],
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                    identifier: "Things".into(),
                    constraints: vec![]
                }))
            })
        );
    }

    #[test]
    fn parses_constraint_sequence_of_elsewhere_declared_type() {
        assert_eq!(
            sequence_of("SEQUENCE SIZE (1..13,...) OF CorrelationCellValue  ")
                .unwrap()
                .1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(1)),
                            max: Some(ASN1Value::Integer(13)),
                            extensible: true
                        })
                    ))),
                    extensible: false
                })],
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                    identifier: "CorrelationCellValue".into(),
                    constraints: vec![]
                }))
            })
        );
    }

    #[test]
    fn parses_constraint_sequence_of_with_extra_parentheses() {
        assert_eq!(
            sequence_of("SEQUENCE (SIZE (1..13, ...)) OF CorrelationCellValue  ")
                .unwrap()
                .1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(1)),
                            max: Some(ASN1Value::Integer(13)),
                            extensible: true
                        })
                    ))),
                    extensible: false
                })],
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                    identifier: "CorrelationCellValue".into(),
                    constraints: vec![]
                }))
            })
        );
    }

    #[test]
    fn parses_constraint_sequence_of_constraint_integer() {
        assert_eq!(
            sequence_of(
                r#"SEQUENCE SIZE (1..13,...) OF INTEGER {
              one-distinguished-value (12)
            } (1..13,...) "#
            )
            .unwrap()
            .1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(1)),
                            max: Some(ASN1Value::Integer(13)),
                            extensible: true
                        })
                    ))),
                    extensible: false
                })],
                r#type: Box::new(ASN1Type::Integer(Integer {
                    constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                        set: ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(1)),
                            max: Some(ASN1Value::Integer(13)),
                            extensible: true
                        }),
                        extensible: false
                    })],
                    used_in_const: false,
                    distinguished_values: Some(vec![DistinguishedValue {
                        name: "one-distinguished-value".into(),
                        value: 12
                    }])
                }))
            })
        );
    }

    #[test]
    fn parses_parameterized_constrained_sequence_of() {
        assert_eq!(
            sequence_of(
                r#"SEQUENCE (SIZE(1..4)) OF 
      RegionalExtension {{Reg-MapData}} OPTIONAL,"#
            )
            .unwrap()
            .1,
            ASN1Type::SequenceOf(SequenceOrSetOf {
                constraints: vec![Constraint::SubtypeConstraint(ElementSet {
                    set: ElementOrSetOperation::Element(SubtypeElement::SizeConstraint(Box::new(
                        ElementOrSetOperation::Element(SubtypeElement::ValueRange {
                            min: Some(ASN1Value::Integer(1)),
                            max: Some(ASN1Value::Integer(4)),
                            extensible: false
                        })
                    ))),
                    extensible: false
                })],
                r#type: Box::new(ASN1Type::ElsewhereDeclaredType(DeclarationElsewhere {
                    identifier: "RegionalExtension".into(),
                    constraints: vec![Constraint::Parameter(vec![Parameter::ObjectSetParameter(
                        ObjectSet {
                            values: vec![ObjectSetValue::Reference("Reg-MapData".into())],
                            extensible: None
                        }
                    )])]
                }))
            })
        );
    }

    #[test]
    fn handles_object_field_ref() {
        println!(
            "{:?}",
            sequence_of(
                r#"SEQUENCE (SIZE(1..MAX)) OF
        IEEE1609DOT2-HEADERINFO-CONTRIBUTED-EXTENSION.&Extn({
        Ieee1609Dot2HeaderInfoContributedExtensions
      }{@.contributorId})"#
            )
            .unwrap()
            .1
        )
    }
}
