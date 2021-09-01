use crate::INSTANCE;
use bytes::Bytes;
pub use misskey::model::user::User as ApiUser;

#[derive(Debug, Clone)]
pub struct User {
    pub display_name: String,
    pub handle: UserHandle,
    pub avatar: UserAvatar,
}

pub async fn import_user(api_user: &ApiUser) -> User {
    let display_name = if let Some(display_name) = api_user.name.clone() {
        display_name
    } else {
        "unnamed".into()
    };

    let handle = import_user_handle(&api_user);
    let avatar = import_user_avatar(&api_user);

    User {
        avatar,
        display_name,
        handle,
    }
}

#[derive(Debug, Clone)]
pub enum UserHandle {
    Local(String, String),
    Remote(String, String),
}

impl std::fmt::Display for UserHandle {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserHandle::Local(name, instance) | UserHandle::Remote(name, instance) => {
                fmt.write_str(&name)?;
                fmt.write_str("@")?;
                fmt.write_str(&instance)
            }
        }
    }
}

pub fn import_user_handle(api_user: &ApiUser) -> UserHandle {
    if let Some(host) = &api_user.host {
        if host == INSTANCE {
            UserHandle::Local(api_user.username.clone(), INSTANCE.into())
        } else {
            UserHandle::Remote(api_user.username.clone(), host.clone())
        }
    } else {
        UserHandle::Local(api_user.username.clone(), INSTANCE.into())
    }
}

#[derive(Debug, Clone)]
pub enum UserAvatar {
    NotSet,
    Fetched(String, Bytes),
    NotFetched(String),
}

pub fn import_user_avatar(api_user: &ApiUser) -> UserAvatar {
    if let Some(url) = &api_user.avatar_url {
        UserAvatar::NotFetched(url.to_string())
    } else {
        UserAvatar::NotSet
    }
}
