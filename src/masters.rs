use std::str::FromStr;

#[derive(Debug, strum::Display, strum::EnumIter, strum::EnumMessage, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Master {
    #[strum(
        message = "Benjamin Graham",
        serialize = "graham",
        serialize = "ben-graham"
    )]
    BenjaminGraham,

    #[strum(
        message = "Warren Buffett",
        serialize = "buffett",
        serialize = "warren-buffett"
    )]
    WarrenBuffett,
}

pub fn parse_masters(s: &str) -> Result<Vec<Master>, strum::ParseError> {
    s.split(',')
        .map(|s| s.trim())
        .map(Master::from_str)
        .collect()
}
