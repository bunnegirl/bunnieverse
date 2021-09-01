use crate::gui::{AppModel, AppMsg};
use bytes::Bytes;
use relm4::*;
use relm4::{Model, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct HttpModel {
    client: reqwest::Client,
    avatars: Arc<Mutex<HashMap<String, Option<Bytes>>>>,
}

impl Model for HttpModel {
    type Msg = HttpMsg;
    type Widgets = ();
    type Components = ();
}

pub enum HttpMsg {
    GetAvatar(usize, String),
    SetAvatar(usize, String, Bytes),
}

#[relm4::async_trait]
impl AsyncComponentUpdate<AppModel> for HttpModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        HttpModel {
            client: reqwest::Client::new(),
            avatars: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn update(
        &mut self,
        msg: HttpMsg,
        _components: &(),
        _sender: Sender<HttpMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            HttpMsg::GetAvatar(index, url) => {
                let client = self.client.clone();
                let avatars = self.avatars.clone();

                tokio::spawn(async move {
                    let mut avatars = avatars.lock().await;

                    if let None = avatars.get(&url) {
                        let bytes = if let Ok(req) = client.get(&url).send().await {
                            req.bytes().await.ok()
                        } else {
                            None
                        };

                        avatars.insert(url.clone(), bytes.clone());
                    }

                    if let Some(Some(bytes)) = avatars.get(&url) {
                        parent_sender
                            .send(AppMsg::HttpResponse(HttpMsg::SetAvatar(
                                index,
                                url,
                                (*bytes).clone(),
                            )))
                            .unwrap();
                    }
                })
                .await
                .unwrap();
            }
            _ => {}
        }
    }
}
