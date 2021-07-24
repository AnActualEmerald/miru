use std::fmt::Display;

pub enum AnimeField {
    ID,
    Title,
    MainPicture,
    AlternativeTitles,
    StartDate,
    EndDate,
    Synopsis,
    Mean,
    Rank,
    Popularity,
    NumListUsers,
    NumScoringUsers,
    NSFW,
    CreatedAt,
    UpdatedAt,
    MediaType,
    Status,
    Genres,
    MyListStatus,
    NumEpisodes,
    StartSeason,
    Broadcast,
    Source,
    AverageEpisodeDuration,
    Rating,
    Pictures,
    Background,
    RelatedAnime,
    RelatedManga,
    Recommendations,
    Studios,
    Statistics,
    ALL,
}

impl AnimeField {
    pub fn to_string(&self) -> String {
        match self {
            
    Self::ID => "id".to_owned(),
    Self::Title => "title".to_owned(),
    Self::MainPicture => "main_picture".to_owned(),
    Self::AlternativeTitles => "alternative_titles".to_owned(),
    Self::StartDate => "start_date".to_owned(),
    Self::EndDate => "end_date".to_owned(),
    Self::Synopsis => "synopsis".to_owned(),
    Self::Mean => "mean".to_owned(),
    Self::Rank => "rank".to_owned(),
    Self::Popularity => "popularity".to_owned(),
    Self::NumListUsers => "num_list_users".to_owned(),
    Self::NumScoringUsers => "num_scoring_users".to_owned(),
    Self::NSFW => "nsfw".to_owned(),
    Self::CreatedAt => "created_at".to_owned(),
    Self::UpdatedAt => "updated_at".to_owned(),
    Self::MediaType => "media_type".to_owned(),
    Self::Status => "status".to_owned(),
    Self::Genres => "genres".to_owned(),
    Self::MyListStatus => "my_list_status".to_owned(),
    Self::NumEpisodes => "num_episodes".to_owned(),
    Self::StartSeason => "start_season".to_owned(),
    Self::Broadcast => "broadcast".to_owned(),
    Self::Source => "source".to_owned(),
    Self::AverageEpisodeDuration => "average_episode_duration".to_owned(),
    Self::Rating => "rating".to_owned(),
    Self::Pictures => "pictures".to_owned(),
    Self::Background => "background".to_owned(),
    Self::RelatedAnime => "related_anime".to_owned(),
    Self::RelatedManga => "related_manga".to_owned(),
    Self::Recommendations => "recommendations".to_owned(),
    Self::Studios => "studios".to_owned(),
    Self::Statistics => "statistics".to_owned(),
    Self::ALL => "id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,created_at,updated_at,media_type,status,genres,my_list_status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,pictures,background,related_anime,related_manga,recommendations,studios,statistics".to_owned(),
        }
    }
}

impl Display for AnimeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
