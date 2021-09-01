use crate::INSTANCE;
use bytes::Bytes;
use log::debug;
pub use misskey::model::user::User as ApiUser;

#[derive(Debug, Clone)]
pub struct User {
    pub display_name: String,
    pub handle: UserHandle,
    pub avatar: UserAvatar,
}

pub fn import_user(api_user: &ApiUser) -> User {
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

// https://blob.jortage.com/blobs/4/d92/4d92366c52c5755bf821961adaea6e89cfdbca7741e4b082f3a9f01e266f412630d5f85e164c2577733d0b18214231a4ae216c1e9d2245c117e0f143df808f38
// https://pool.jortage.com/pluralcafe/accounts/avatars/000/113/478/original/1950ffd6168e5056.png

impl UserAvatar {
    pub async fn from_response(url: String, response: Option<surf::Response>) -> Self {
        debug!("get avatar {}", url);

        if let Some(mut req) = response {
            if let Ok(body) = req.body_bytes().await {
                debug!("got avatar {} = {} ({})", url, body.len(), req.status());
                UserAvatar::Fetched(url, Bytes::from(body))
            } else {
                debug!("no avatar {} ({})", url, req.status());
                UserAvatar::NotSet
            }
        } else {
            debug!("no avatar {}", url);
            UserAvatar::NotSet
        }
    }
}

pub fn import_user_avatar(api_user: &ApiUser) -> UserAvatar {
    if let Some(url) = &api_user.avatar_url {
        UserAvatar::NotFetched(url.to_string())
    } else {
        UserAvatar::NotSet
    }
}
