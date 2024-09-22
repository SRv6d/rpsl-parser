use std::iter::once;

use winnow::{
    ascii::{newline, space0},
    combinator::{delimited, peek, preceded, repeat, separated_pair, terminated},
    error::{ContextError, ParserError},
    stream::ContainsToken,
    token::{one_of, take_while},
    PResult, Parser,
};

use crate::Attribute;

const ATTR_NAME_SET: (
    std::ops::RangeInclusive<char>,
    std::ops::RangeInclusive<char>,
    std::ops::RangeInclusive<char>,
    char,
    char,
) = ('A'..='Z', 'a'..='z', '0'..='9', '-', '_');

// A response code or message sent by the whois server.
// Starts with the "%" character and extends until the end of the line.
// In contrast to RPSL, characters are not limited to ASCII.
pub fn server_message<'s>(input: &mut &'s str) -> PResult<&'s str> {
    delimited(
        ('%', space0),
        take_while(0.., |c: char| !c.is_control()),
        newline,
    )
    .parse_next(input)
}

// A RPSL attribute consisting of a name and one or more values.
// The name is followed by a colon and optional spaces.
// Single value attributes are limited to one line, while multi value attributes span over multiple lines.
pub fn attribute<'s>(input: &mut &'s str) -> PResult<Attribute<'s>> {
    let (name, first_value) = separated_pair(
        terminated(attribute_name(ATTR_NAME_SET), ':'),
        space0,
        terminated(
            attribute_value(|c: char| c.is_ascii() && !c.is_ascii_control()),
            newline,
        ),
    )
    .parse_next(input)?;

    if peek(continuation_char::<ContextError>())
        .parse_next(input)
        .is_ok()
    {
        let continuation_values: Vec<&str> = repeat(
            1..,
            continuation_line(attribute_value(|c: char| {
                c.is_ascii() && !c.is_ascii_control()
            })),
        )
        .parse_next(input)?;
        return Ok(Attribute::unchecked_multi(
            name,
            once(first_value).chain(continuation_values),
        ));
    }

    Ok(Attribute::unchecked_single(name, first_value))
}

/// Generate an attribute value parser given a set of valid chars.
/// The first character must be a letter, while the last character may be a letter or a digit.
fn attribute_name<'s, S, E>(set: S) -> impl Parser<&'s str, &'s str, E>
where
    S: ContainsToken<char>,
    E: ParserError<&'s str>,
{
    take_while(2.., set).verify(|s: &str| {
        s.starts_with(|c: char| c.is_ascii_alphabetic())
            && s.ends_with(|c: char| c.is_ascii_alphanumeric())
    })
}

/// Generate an attribute value parser given a set of valid chars.
fn attribute_value<'s, S, E>(set: S) -> impl Parser<&'s str, &'s str, E>
where
    S: ContainsToken<char>,
    E: ParserError<&'s str>,
{
    take_while(0.., set)
}

/// Generate a parser that extends an attributes value over multiple lines,
/// where each value is prefixed with a continuation character.
fn continuation_line<'s, P, E>(value_parser: P) -> impl Parser<&'s str, &'s str, E>
where
    P: Parser<&'s str, &'s str, E>,
    E: ParserError<&'s str>,
{
    delimited(continuation_char(), preceded(space0, value_parser), newline)
}

/// Generate a parser for a single continuation character.
fn continuation_char<'s, E>() -> impl Parser<&'s str, char, E>
where
    E: ParserError<&'s str>,
{
    one_of([' ', '\t', '+'])
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(
        &mut "% Note: this output has been filtered.\n",
        "Note: this output has been filtered.",
        ""
    )]
    #[case(
        &mut "%       To receive output for a database update, use the \"-B\" flag.\n",
        "To receive output for a database update, use the \"-B\" flag.",
        ""
    )]
    #[case(
        &mut "% This query was served by the RIPE Database Query Service version 1.106.1 (BUSA)\n",
        "This query was served by the RIPE Database Query Service version 1.106.1 (BUSA)",
        ""
    )]
    fn server_message_valid(
        #[case] given: &mut &str,
        #[case] expected: &str,
        #[case] remaining: &str,
    ) {
        let parsed = server_message(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }

    #[rstest]
    #[case(
        &mut "import:         from AS12 accept AS12\n",
        Attribute::unchecked_single("import", "from AS12 accept AS12"),
        ""
    )]
    fn attribute_valid_single_value(
        #[case] given: &mut &str,
        #[case] expected: Attribute,
        #[case] remaining: &str,
    ) {
        let parsed = attribute(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }

    #[rstest]
    #[case(
        &mut concat!(
            "remarks:        Locations\n",
            "                LA1 - CoreSite One Wilshire\n",
            "                NY1 - Equinix New York, Newark\n",
            "remarks:        Peering Policy\n",
        ),
        Attribute::unchecked_multi(
            "remarks",
            vec![
                "Locations",
                "LA1 - CoreSite One Wilshire",
                "NY1 - Equinix New York, Newark",
            ]
        ),
        "remarks:        Peering Policy\n"
    )]
    fn attribute_valid_multi_value(
        #[case] given: &mut &str,
        #[case] expected: Attribute,
        #[case] remaining: &str,
    ) {
        let parsed = attribute(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }

    #[rstest]
    #[case(ATTR_NAME_SET, &mut "remarks:", "remarks", ":")]
    #[case(ATTR_NAME_SET, &mut "aut-num:", "aut-num", ":")]
    #[case(ATTR_NAME_SET, &mut "ASNumber:", "ASNumber", ":")]
    #[case(ATTR_NAME_SET, &mut "route6:", "route6", ":")]
    fn attribute_name_valid(
        #[case] set: impl ContainsToken<char>,
        #[case] given: &mut &str,
        #[case] expected: &str,
        #[case] remaining: &str,
    ) {
        let mut parser = attribute_name::<_, ContextError>(set);
        let parsed = parser.parse_next(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }

    #[rstest]
    #[case(ATTR_NAME_SET, &mut "1remarks:")]
    #[case(ATTR_NAME_SET, &mut "-remarks:")]
    #[case(ATTR_NAME_SET, &mut "_remarks:")]
    fn attribute_name_non_letter_first_char_is_error(
        #[case] set: impl ContainsToken<char>,
        #[case] given: &mut &str,
    ) {
        let mut parser = attribute_name::<_, ContextError>(set);
        assert!(parser.parse_next(given).is_err());
    }

    #[rstest]
    #[case(ATTR_NAME_SET, &mut "remarks-:")]
    #[case(ATTR_NAME_SET, &mut "remarks_:")]
    fn attribute_name_non_letter_or_digit_last_char_is_error(
        #[case] set: impl ContainsToken<char>,
        #[case] given: &mut &str,
    ) {
        let mut parser = attribute_name::<_, ContextError>(set);
        assert!(parser.parse_next(given).is_err());
    }

    #[test]
    fn attribute_name_single_letter_is_error() {
        let mut parser = attribute_name::<_, ContextError>(ATTR_NAME_SET);
        assert!(parser.parse_next(&mut "a").is_err());
    }

    #[rstest]
    #[case(
            |c: char| c.is_ascii() && !c.is_ascii_control(),
            &mut "This is an example remark\n",
            "This is an example remark",
            "\n"
        )]
    #[case(
            |c: char| c.is_ascii() && !c.is_ascii_control(),
            &mut "Concerning abuse and spam ... mailto: abuse@asn.net\n",
            "Concerning abuse and spam ... mailto: abuse@asn.net",
            "\n"
        )]
    #[case(
            |c: char| c.is_ascii() && !c.is_ascii_control(),
            &mut "+49 176 07071964\n",
            "+49 176 07071964",
            "\n"
        )]
    #[case(
            |c: char| c.is_ascii() && !c.is_ascii_control(),
            &mut "* Equinix FR5, Kleyerstr, Frankfurt am Main\n",
            "* Equinix FR5, Kleyerstr, Frankfurt am Main",
            "\n"
        )]
    #[case(
            |c: char| c.is_ascii() && !c.is_ascii_control(),
            &mut "\n",
            "",
            "\n"
        )]
    fn attribute_value_valid(
        #[case] set: impl ContainsToken<char>,
        #[case] given: &mut &str,
        #[case] expected: &str,
        #[case] remaining: &str,
    ) {
        let mut parser = attribute_value::<_, ContextError>(set);
        let parsed = parser.parse_next(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }

    #[rstest]
    #[case(
            &mut "    continuation value prefixed by a space\n",
            "continuation value prefixed by a space",
            ""
        )]
    #[case(
            &mut "\t    continuation value prefixed by a tab\n",
            "continuation value prefixed by a tab",
            ""
        )]
    #[case(
            &mut "+    continuation value prefixed by a plus\n",
            "continuation value prefixed by a plus",
            ""
        )]
    fn continuation_line_valid(
        #[case] given: &mut &str,
        #[case] expected: &str,
        #[case] remaining: &str,
    ) {
        let mut parser = continuation_line::<_, ContextError>(attribute_value(|c: char| {
            c.is_ascii() && !c.is_ascii_control()
        }));
        let parsed = parser.parse_next(given).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(*given, remaining);
    }
}
