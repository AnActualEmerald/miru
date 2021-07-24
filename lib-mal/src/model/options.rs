use std::fmt::Display;

#[derive(Debug)]
pub enum RankingType {
    All,
    Airing,
    Upcoming,
    TV,
    OVA,
    Movie,
    Special,
    ByPopularity,
    Favorite,
}

impl RankingType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Favorite => "favorite".to_owned(),
            Self::TV => "tv".to_owned(),
            Self::Airing => "airing".to_owned(),
            Self::Upcoming => "upcoming".to_owned(),
            Self::Special => "special".to_owned(),
            Self::ByPopularity => "bypopularity".to_owned(),
            Self::Movie => "movie".to_owned(),
            Self::OVA => "ova".to_owned(),
            Self::All => "all".to_owned(),
        }
    }
}

impl Display for RankingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
