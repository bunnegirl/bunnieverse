use crate::model::note::import_note;
use crate::timeline::TimelineMsg;
use const_env::from_env;
use misskey::{prelude::*, WebSocketClient};
use relm4::Sender;

#[from_env]
const API_WS: &str = "";
#[from_env]
const API_TOKEN: &str = "";

#[tokio::main]
async fn begin(sender: Sender<TimelineMsg>) {
    let err_sender = sender.clone();

    preload_home_timeline(sender).await.unwrap_or_else(|_| {
        err_sender.send(TimelineMsg::ConnectionError).unwrap();
    });
}

async fn preload_home_timeline(sender: Sender<TimelineMsg>) -> anyhow::Result<()> {
    println!("start");

    let client = WebSocketClient::builder(API_WS)
        .token(API_TOKEN)
        .connect()
        .await?;

    // use std::str::FromStr;

    // let id = misskey::model::id::Id::<misskey::model::note::Note>::from_str("8q3jt2i0hb").unwrap();

    // let note = client.get_note(id).await.unwrap();

    // sender
    //     .send(TimelineMsg::InsertNote(import_note(note, true).await))
    //     .unwrap();

    use futures::stream::{StreamExt, TryStreamExt};

    let mut timeline = client.home_notes(..).take(20);

    while let Some(note) = timeline.try_next().await? {
        sender
            .send(TimelineMsg::InsertNote(import_note(note, true).await))
            .unwrap();
    }

    println!("done");

    Ok(())
}

pub fn thread(sender: Sender<TimelineMsg>) {
    begin(sender);
}
