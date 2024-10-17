use sea_orm::{EnumIter, Iterable};

/// notify_client_template會使用user_profile 來取代 title 和 content 中的 placeholder
/// 這是定義所有的placeholder key的地方
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum CommonKey {
    UserAccount = 1,
    UserLastName = 2,
    UserFirstName = 3,
    UserCity = 4,
    UserCountry = 5,
}

impl CommonKey {
    pub fn get_key(&self) -> &str {
        match self {
            Self::UserAccount => "{{user_account}}",
            Self::UserLastName => "{{user_last_name}}",
            Self::UserFirstName => "{{user_first_name}}",
            Self::UserCity => "{{user_city}}",
            Self::UserCountry => "{{user_country}}",
        }
    }

    pub fn get_all_keys() -> Vec<String> {
        CommonKey::iter().map(|x| x.get_key().to_string()).collect()
    }
}
