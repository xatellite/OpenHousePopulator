use itertools::Itertools;
use nom::{
    branch::alt,
    character::complete::{alpha0, char, digit1, multispace0},
    combinator::complete,
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    IResult,
};
use std::{collections::HashSet, error, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HouseNumber {
    Single(SingleHouseNumber),
    Range(SingleHouseNumber, SingleHouseNumber),
}

impl HouseNumber {
    pub fn count(&self) -> u16 {
        match self {
            HouseNumber::Single(_) => 1,
            HouseNumber::Range(hn1, hn2) => hn2.0 - hn1.0,
        }
    }

    fn singles(&self) -> Vec<SingleHouseNumber> {
        match self {
            HouseNumber::Single(hn) => vec![hn.to_owned()],
            HouseNumber::Range(hn1, hn2) => (hn1.0..hn2.0)
                .map(|e| SingleHouseNumber(e, String::new()))
                .collect(),
        }
    }
}

impl Display for HouseNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HouseNumber::Single(hn) => write!(f, "{}", hn),
            HouseNumber::Range(hn1, hn2) => write!(f, "{}-{}", hn1, hn2),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct SingleHouseNumber(u16, String);

impl Display for SingleHouseNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

#[derive(Default, Debug)]
pub struct HouseNumberList(HashSet<SingleHouseNumber>);

impl HouseNumberList {
    pub fn merge(&mut self, other: HouseNumberList) {
        self.0.extend(other.0);
    }

    pub fn new() -> Self {
        HouseNumberList(HashSet::new())
    }
}

impl HouseNumberList {
    pub fn count(&self) -> usize {
        self.0.len()
    }
}

impl Display for HouseNumberList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|e| e.to_string()).sorted().join("/")
        )
    }
}

impl<'a> TryFrom<&'a str> for HouseNumberList {
    type Error = ParseError<'a>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        let (rest, value) = housenumber_list(input).map_err(ParseError::NomError)?;
        if !rest.is_empty() {
            return Err(ParseError::NotFullyConsumedError(input, rest));
        }
        Ok(value)
    }
}

#[derive(Debug)]
pub enum ParseError<'a> {
    NotFullyConsumedError(&'a str, &'a str),
    NomError(nom::Err<nom::error::Error<&'a str>>),
}

impl<'a> error::Error for ParseError<'a> {}
impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotFullyConsumedError(input, rest) => {
                write!(f, "Parsing failed for string {}, with rest {}", input, rest)
            }
            ParseError::NomError(err) => write!(f, "Parsing failed with NomError: {}", err),
        }
    }
}

pub fn house_number(input: &str) -> IResult<&str, HouseNumber> {
    alt((complete(housenumber_range), wrapped_housenumber))(input)
}

pub fn wrapped_housenumber(input: &str) -> IResult<&str, HouseNumber> {
    let (rest, housenumber) = single_housenumber(input)?;
    Ok((rest, HouseNumber::Single(housenumber)))
}

pub fn single_housenumber(input: &str) -> IResult<&str, SingleHouseNumber> {
    let (rest, (number, letter)) = pair(digit1, ws(alpha0))(input)?;
    Ok((
        rest,
        SingleHouseNumber(number.parse().unwrap_or_default(), letter.to_string()),
    ))
}

pub fn housenumber_range(input: &str) -> IResult<&str, HouseNumber> {
    let (rest, (housenumber1, housenumber2)) =
        separated_pair(single_housenumber, ws(char('-')), single_housenumber)(input)?;
    Ok((rest, HouseNumber::Range(housenumber1, housenumber2)))
}

pub fn housenumber_list(input: &str) -> IResult<&str, HouseNumberList> {
    let (rest, list) = separated_list1(list_delimiter, house_number)(input)?;
    Ok((
        rest,
        HouseNumberList(list.into_iter().flat_map(|e| e.singles()).collect()),
    ))
}

pub fn list_delimiter(input: &str) -> IResult<&str, char> {
    alt((ws(char('/')), ws(char(',')), ws(char(';'))))(input)
}

fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
