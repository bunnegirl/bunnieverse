use crate::api;
use crate::gui::*;
use crate::http::HttpMsg;
use crate::model::note::*;
use crate::model::user::*;
use bytes::Bytes;
use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, ListBoxRowExt};
use gtk::Align;
use log::error;
use pango::{EllipsizeMode, WrapMode};
use relm4::factory::{Factory, FactoryPrototype, FactoryVec};
use relm4::*;
use relm4::{Model, Sender, Widgets};

pub struct TimelineModel {
    notes: FactoryVec<Note>,
}

pub enum TimelineMsg {
    InsertNote(Note),
    HttpRequest(HttpMsg),
    HttpResponse(HttpMsg),
}

// pub enum TimelineType {
//     Home,
//     // Local,
//     // Global,
// }

impl ComponentUpdate<AppModel> for TimelineModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        TimelineModel {
            notes: FactoryVec::new(),
            // timeline: TimelineType::Home,
        }
    }

    fn update(
        &mut self,
        msg: TimelineMsg,
        _components: &TimelineComponents,
        _sender: Sender<TimelineMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        use TimelineMsg::*;

        match msg {
            InsertNote(note) => {
                self.notes.push(note);
            }
            HttpRequest(msg) => {
                parent_sender.send(AppMsg::HttpRequest(msg)).unwrap();
            }
            HttpResponse(msg) => {
                if let HttpMsg::SetAvatar(index, key, bytes) = msg {
                    if let Some(note) = self.notes.get_mut(index) {
                        if let UserAvatar::NotFetched(url) = &note.user.avatar {
                            if url == &key {
                                note.user.avatar = UserAvatar::Fetched(url.clone(), bytes.clone());
                            }
                        }

                        if let Some(child) = &mut note.note {
                            if let UserAvatar::NotFetched(url) = &child.user.avatar {
                                if url == &key {
                                    child.user.avatar = UserAvatar::Fetched(url.clone(), bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Model for TimelineModel {
    type Msg = TimelineMsg;
    type Widgets = TimelineWidgets;
    type Components = TimelineComponents;
}

pub struct TimelineComponents {}

impl Components<TimelineModel> for TimelineComponents {
    fn init_components(
        _parent_model: &TimelineModel,
        _parent_widgets: &TimelineWidgets,
        _parent_sender: Sender<TimelineMsg>,
    ) -> Self {
        TimelineComponents {}
    }
}

pub struct TimelineWidgets {
    notes: gtk::ListBox,
    frame: gtk::Frame,
}

impl Widgets<TimelineModel, AppModel> for TimelineWidgets {
    type Root = gtk::Frame;

    fn init_view(
        _model: &TimelineModel,
        _parent_widgets: &AppWidgets,
        sender: Sender<TimelineMsg>,
    ) -> Self {
        let frame = gtk::Frame::builder()
            .margin_bottom(20)
            .margin_top(20)
            .margin_end(20)
            .margin_start(20)
            .css_classes(vec!["timeline".into()])
            .build();

        let notes = gtk::ListBox::new();

        frame.set_child(Some(&notes));

        std::thread::spawn(move || {
            api::thread(sender);
        });

        TimelineWidgets { notes, frame }
    }

    fn view(&mut self, model: &TimelineModel, sender: Sender<TimelineMsg>) {
        model.notes.generate(&self.notes, sender);
    }

    fn root_widget(&self) -> gtk::Frame {
        self.frame.clone()
    }
}

pub struct TimelineFactory {
    root: gtk::ListBoxRow,
    avatars: Vec<(String, gtk::Image)>,
}

impl FactoryPrototype for Note {
    type Factory = FactoryVec<Self>;
    type Widgets = TimelineFactory;
    type Root = gtk::ListBoxRow;
    type View = gtk::ListBox;
    type Msg = TimelineMsg;

    fn generate(&self, index: &usize, sender: Sender<TimelineMsg>) -> TimelineFactory {
        let root = gtk::ListBoxRow::builder().selectable(false).build();
        let (note, avatars) = note(sender.clone(), *index, &self);

        root.set_child(Some(&note));

        TimelineFactory { avatars, root }
    }

    fn position(&self, _index: &usize) {}

    fn update(&self, _index: &usize, widgets: &TimelineFactory) {
        for (key, avatar) in &widgets.avatars {
            if let UserAvatar::Fetched(url, bytes) = &self.user.avatar {
                if url == key {
                    set_image_from_bytes(&url, bytes, avatar);
                }
            }
        }
    }

    fn get_root(widgets: &TimelineFactory) -> &gtk::ListBoxRow {
        &widgets.root
    }
}

fn set_image_from_bytes(url: &str, bytes: &Bytes, image: &gtk::Image) {
    let bytes = glib::Bytes::from(&bytes.to_vec());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    match gdk_pixbuf::Pixbuf::from_stream(&stream, gio::NONE_CANCELLABLE) {
        Ok(pixbuf) => image.set_from_pixbuf(Some(&pixbuf)),
        Err(_) => {
            error!("error reading image {}:", url);
        }
    }
}

fn note(
    sender: Sender<TimelineMsg>,
    index: usize,
    note: &Note,
) -> (gtk::Grid, Vec<(String, gtk::Image)>) {
    let grid = gtk::Grid::builder()
        .column_spacing(8)
        .margin_end(16)
        .margin_start(16)
        .margin_bottom(8)
        .margin_top(8)
        .build();

    let (mut avatars, avatar_button, display_name, handle) =
        note_header(sender.clone(), index, &note);
    let (content, content_avatars) = note_content(sender.clone(), index, &note);

    let reply_button = gtk::Button::builder()
        .icon_name("mail-reply-sender-symbolic")
        .build();

    let renote_button = gtk::Button::builder()
        .icon_name("media-playlist-repeat-symbolic")
        .build();

    let react_button = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .build();

    let more_button = gtk::Button::builder()
        .icon_name("view-more-symbolic")
        .halign(Align::End)
        .hexpand(true)
        .build();

    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(8)
        .spacing(8)
        .build();

    buttons.append(&reply_button);
    buttons.append(&renote_button);
    buttons.append(&react_button);
    buttons.append(&more_button);

    let note_type = gtk::Label::builder()
        .label(match note.kind {
            NoteKind::Note => "note",
            NoteKind::Renote => "renote",
            NoteKind::Reply => "renote",
        })
        .halign(Align::End)
        .build();

    grid.attach(&avatar_button, 1, 1, 1, 2);
    grid.attach(&display_name, 2, 1, 2, 1);
    grid.attach(&handle, 2, 2, 2, 1);
    grid.attach(&note_type, 3, 1, 2, 1);
    grid.attach(&content, 2, 3, 2, 1);
    grid.attach(&buttons, 2, 4, 2, 1);

    avatars.extend(content_avatars);

    (grid, avatars)
}

fn renote(
    sender: Sender<TimelineMsg>,
    index: usize,
    note: &Note,
) -> (gtk::Button, Vec<(String, gtk::Image)>) {
    let frame = gtk::Button::builder()
        .margin_bottom(8)
        .margin_top(8)
        .hexpand(true)
        .css_classes(vec!["note".into()])
        .build();

    let grid = gtk::Grid::builder()
        .column_spacing(8)
        .margin_end(16)
        .margin_start(16)
        .margin_bottom(16)
        .margin_top(16)
        .build();

    let (mut avatars, avatar_button, display_name, handle) =
        note_header(sender.clone(), index, &note);
    let (content, content_avatars) = note_content(sender.clone(), index, &note);

    grid.attach(&avatar_button, 1, 1, 1, 2);
    grid.attach(&display_name, 2, 1, 2, 1);
    grid.attach(&handle, 2, 2, 2, 1);
    grid.attach(&content, 2, 3, 2, 1);

    frame.set_child(Some(&grid));

    avatars.extend(content_avatars);

    (frame, avatars)
}

fn note_header(
    sender: Sender<TimelineMsg>,
    index: usize,
    note: &Note,
) -> (
    Vec<(String, gtk::Image)>,
    gtk::Button,
    gtk::Label,
    gtk::Label,
) {
    let avatar_image = gtk::Image::builder()
        .icon_name("avatar-default-symbolic")
        .icon_size(gtk::IconSize::Large)
        .build();

    let mut avatars = Vec::new();

    if let UserAvatar::NotFetched(url) = &note.user.avatar {
        let url = url.clone();
        let index = index.clone();

        avatars.push((url.clone(), avatar_image.clone()));

        spawn_future(async move {
            sender
                .send(TimelineMsg::HttpRequest(HttpMsg::GetAvatar(index, url)))
                .unwrap();
        });
    }

    let avatar_button = gtk::Button::builder()
        .height_request(48)
        .width_request(48)
        .css_classes(vec![
            "flat".into(),
            "circular".into(),
            "image-button".into(),
        ])
        .build();

    avatar_button.set_child(Some(&avatar_image));

    let display_name = gtk::Label::builder()
        .label(&note.user.display_name)
        .ellipsize(EllipsizeMode::End)
        .halign(Align::Start)
        .css_classes(vec!["title-4".into()])
        .build();

    let handle = gtk::Label::builder()
        .label(&note.user.handle.to_string())
        .ellipsize(EllipsizeMode::End)
        .halign(Align::Start)
        .css_classes(vec!["body".into()])
        .build();

    (avatars, avatar_button, display_name, handle)
}

fn note_content(
    sender: Sender<TimelineMsg>,
    index: usize,
    note: &Note,
) -> (gtk::Box, Vec<(String, gtk::Image)>) {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    let mut avatars = Vec::new();

    if let Some(text) = &note.body {
        let body = gtk::Label::builder()
            .halign(Align::Start)
            .margin_top(8)
            .wrap(true)
            .wrap_mode(WrapMode::WordChar)
            .xalign(0.0)
            .use_markup(true)
            .label(text)
            .css_classes(vec!["body".into()])
            .build();

        if let Some(cw) = &note.cw {
            let expander = gtk::Expander::builder().margin_top(8).label(&cw).build();

            expander.set_child(Some(&body));
            content.append(&expander);
        } else {
            content.append(&body);
        };
    }

    if let Some(child) = &note.note {
        let (renote, renote_avatars) = renote(sender.clone(), index, &(**child));

        avatars.extend(renote_avatars);
        content.append(&renote);
    }

    (content, avatars)
}
