use std::{fmt::Display, error};
use nom::{IResult, branch::alt, character::{complete::{digit1, alpha0}, streaming::char}, sequence::{ pair, separated_pair}, multi::{separated_list1}};

pub trait CountableHouseNumber {
  fn count(&self) -> u16;
  fn list_unique(&self) -> Vec<String>;
  fn house_number(&self) -> u16 {
    0
  }
}

struct HouseNumber(u16, String);

impl CountableHouseNumber for HouseNumber {
    fn count(&self) -> u16 {
        1
    }
    fn list_unique(&self) -> Vec<String> {
      let full_text = (self.0.to_string() + self.1.as_str());
      vec![full_text]
    }
    fn house_number(&self) -> u16 {
        self.0
    }
}

struct HouseNumberRange(Box<dyn CountableHouseNumber>, Box<dyn CountableHouseNumber>);

impl CountableHouseNumber for HouseNumberRange {
  fn count(&self) -> u16 {
      self.1.house_number() - self.0.house_number()
  }

  fn list_unique(&self) -> Vec<String> {
    (self.1.house_number()..self.0.house_number()).into_iter().map(|elem| elem.to_string()).collect()
  }
}

#[derive(Default)]
pub struct HouseNumberList (
  Vec<Box<dyn CountableHouseNumber>>
);

impl HouseNumberList {
  // ToDo Test this!
  pub fn merge(&mut self, other: HouseNumberList) {
    self.0.extend(other.0);
    self.0.dedup_by_key(|elem| elem.house_number())
  }

  pub fn new() -> Self {
    HouseNumberList(vec![])
  }
}

impl CountableHouseNumber for HouseNumberList {
  fn count(&self) -> u16 {
    self.0.iter().map(|elem| elem.count()).sum()
  }

  fn list_unique(&self) -> Vec<String> {
    let mut merged = vec![];
    self.0.iter().for_each(|elem| {
      merged.extend(elem.list_unique());
    });
    merged.dedup();
    merged
  }
}

impl<'a> TryFrom<&'a str> for HouseNumberList {
  type Error = ParseError<'a>;

  fn try_from(input: &'a str) -> Result<Self, Self::Error> {
    let (rest, value) = housenumber_list(input).map_err(|err| ParseError::NomError(err))?;
    if rest != "" {
      return Err(ParseError::NotFullyConsumedError(input, rest));
    }
    Ok(value)
  }
}


#[derive(Debug)]
pub enum ParseError<'a> {
  NotFullyConsumedError(&'a str, &'a str),
  NomError(nom::Err<nom::error::Error<&'a str>>)
}

impl<'a> error::Error for ParseError<'a> {}
impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotFullyConsumedError(input, rest) => write!(f, "Parsing failed for string {}, with rest {}", input, rest),
            ParseError::NomError(err) => write!(f, "Parsing failed with NomError: {}", err),
        }
        
    }
}

fn house_number(input: &str) -> IResult<&str, Box<dyn CountableHouseNumber>> {
  alt((housenumber_range, concrete_housenumber))(input)
}

fn concrete_housenumber(input: &str) -> IResult<&str, Box<dyn CountableHouseNumber>> {
  let (rest, (number, letter)) = pair(digit1, alpha0)(input)?;
  Ok((rest, Box::new(HouseNumber(number.parse().unwrap_or_default(), letter.to_string()))))
}

fn housenumber_range(input: &str) -> IResult<&str, Box<dyn CountableHouseNumber>> {
  let (rest, (housenumber1, housenumber2)) = separated_pair(concrete_housenumber, char('-'), concrete_housenumber)(input)?;
  Ok((rest, Box::new(HouseNumberRange(housenumber1, housenumber2))))
}

fn housenumber_list(input: &str) -> IResult<&str, HouseNumberList> {
  let (rest, list) = separated_list1(alt((char('/'), char(','), char(';'))), house_number)(input)?;
  Ok((rest, HouseNumberList(list)))
}

