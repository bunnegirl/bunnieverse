use crate::model::user::*;
use async_recursion::async_recursion;
use log::info;
pub use misskey::model::note::Note as ApiNote;

#[derive(Debug, Clone)]
pub struct Note {
    pub kind: NoteKind,
    pub user: User,
    pub cw: Option<String>,
    pub body: Option<String>,
    pub files: Vec<NoteFile>,
    pub note: Option<Box<Note>>,
}

#[derive(Debug, Clone)]
pub struct NoteFile {}

#[derive(Debug, Clone)]
pub enum NoteKind {
    Note,
    Renote,
    Reply,
}

#[async_recursion]
pub async fn import_note(api_note: ApiNote, recurse: bool) -> Note {
    info!("reading note #{}", api_note.id);

    let user = import_user(&api_note.user);

    let kind = match (
        &api_note.cw,
        &api_note.text,
        &api_note.reply,
        &api_note.renote,
    ) {
        (None, None, None, Some(_)) => NoteKind::Renote,
        (_, Some(_), None, Some(_)) => NoteKind::Reply,
        (_, Some(_), Some(_), None) => NoteKind::Reply,
        _ => NoteKind::Note,
    };

    // Only import child notes when the parent isn't a regular note
    let note = if recurse {
        match (api_note.reply, api_note.renote) {
            (Some(reply), None) => Some(Box::new(import_note(*reply.clone(), false).await)),
            (None, Some(renote)) => Some(Box::new(import_note(*renote.clone(), false).await)),
            _ => None,
        }
    } else {
        None
    };

    let cw = api_note.cw;
    let body = api_note.text;
    let files = Vec::new();

    Note {
        kind,
        user,
        cw,
        body,
        files,
        note,
    }
}
