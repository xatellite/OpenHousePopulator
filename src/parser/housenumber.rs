use itertools::Itertools;
use nom::{
    branch::alt,
    character::complete::{alpha0, alpha1, char, digit1, multispace0},
    combinator::complete,
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    IResult,
};
use std::{collections::HashSet, error, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HouseNumber {
    Single(SingleHouseNumber),
    Subdivided(Subdivisions),
    Range(SingleHouseNumber, SingleHouseNumber),
}

impl HouseNumber {
    fn singles(&self) -> Vec<SingleHouseNumber> {
        match self {
            HouseNumber::Single(hn) => vec![hn.to_owned()],
            HouseNumber::Subdivided(sdv) => sdv.singles(),
            HouseNumber::Range(hn1, hn2) => (hn1.0..hn2.0)
                .map(|e| SingleHouseNumber(e, String::new()))
                .collect(),
        }
    }
}

impl Display for HouseNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HouseNumber::Single(hn) => write!(f, "{hn}"),
            HouseNumber::Subdivided(sdv) => write!(f, "{sdv}"),
            HouseNumber::Range(hn1, hn2) => write!(f, "{hn1}-{hn2}"),
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subdivisions(u16, Vec<String>);

impl Subdivisions {
    fn singles(&self) -> Vec<SingleHouseNumber> {
        self.1
            .iter()
            .map(|e| SingleHouseNumber(self.0, e.clone()))
            .collect()
    }
}

impl Display for Subdivisions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1.join("/"))
    }
}

#[derive(Default, Debug)]
pub struct HouseNumberList(HashSet<SingleHouseNumber>);

impl HouseNumberList {
    pub fn merge(&mut self, other: HouseNumberList) {
        self.0.extend(other.0);
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
                write!(f, "Parsing failed for string {input}, with rest {rest}")
            }
            ParseError::NomError(err) => write!(f, "Parsing failed with NomError: {err}"),
        }
    }
}

fn house_number(input: &str) -> IResult<&str, HouseNumber> {
    alt((complete(housenumber_range), wrapped_housenumber))(input)
}

fn wrapped_housenumber(input: &str) -> IResult<&str, HouseNumber> {
    alt((wrapped_subdivided_housenumber, wrapped_single_housenumber))(input)
}

fn wrapped_single_housenumber(input: &str) -> IResult<&str, HouseNumber> {
    let (rest, housenumber) = single_housenumber(input)?;
    Ok((rest, HouseNumber::Single(housenumber)))
}

fn wrapped_subdivided_housenumber(input: &str) -> IResult<&str, HouseNumber> {
    let (rest, housenumber) = subdivided_housenumber(input)?;
    Ok((rest, HouseNumber::Subdivided(housenumber)))
}

fn subdivided_housenumber(input: &str) -> IResult<&str, Subdivisions> {
    let (rest, (number, letters)) =
        pair(digit1, ws(separated_list1(list_delimiter, alpha1)))(input)?;
    Ok((
        rest,
        Subdivisions(
            number.parse().unwrap_or_default(),
            letters.iter().map(|e| e.to_string()).collect(),
        ),
    ))
}

fn single_housenumber(input: &str) -> IResult<&str, SingleHouseNumber> {
    let (rest, (number, letter)) = pair(digit1, ws(alpha0))(input)?;
    Ok((
        rest,
        SingleHouseNumber(number.parse().unwrap_or_default(), letter.to_string()),
    ))
}

fn housenumber_range(input: &str) -> IResult<&str, HouseNumber> {
    let (rest, (housenumber1, housenumber2)) =
        separated_pair(single_housenumber, ws(char('-')), single_housenumber)(input)?;
    Ok((rest, HouseNumber::Range(housenumber1, housenumber2)))
}

fn housenumber_list(input: &str) -> IResult<&str, HouseNumberList> {
    let (rest, list) = separated_list1(list_delimiter, house_number)(input)?;
    Ok((
        rest,
        HouseNumberList(list.into_iter().flat_map(|e| e.singles()).collect()),
    ))
}

fn list_delimiter(input: &str) -> IResult<&str, char> {
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
