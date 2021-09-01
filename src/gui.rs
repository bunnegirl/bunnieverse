use crate::timeline::*;
use gtk::prelude::GtkWindowExt;
use gtk::PolicyType;
use relm4::*;
use relm4::{AppUpdate, Model, RelmApp, Sender, Widgets};

pub struct AppModel {}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        _msg: AppMsg,
        _components: &AppComponents,
        _sender: Sender<AppMsg>,
    ) -> bool {
        true
    }
}

pub struct AppComponents {
    home_timeline: RelmComponent<TimelineModel, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        parent_model: &AppModel,
        parent_widgets: &AppWidgets,
        parent_sender: Sender<AppMsg>,
    ) -> Self {
        AppComponents {
            home_timeline: RelmComponent::new(parent_model, parent_widgets, parent_sender.clone()),
        }
    }
}

pub enum AppMsg {}

pub struct AppWidgets {
    window: gtk::ApplicationWindow,
    scroll: gtk::ScrolledWindow,
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(_model: &AppModel, _parent_widgets: &(), _sender: Sender<AppMsg>) -> Self {
        let window = gtk::ApplicationWindow::builder()
            .title("bunnieverse")
            .default_width(600)
            .default_height(800)
            .build();

        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .build();

        window.set_child(Some(&scroll));

        AppWidgets { window, scroll }
    }

    fn connect_components(&self, components: &AppComponents) {
        self.scroll
            .set_child(Some(components.home_timeline.root_widget()));
    }

    fn view(&mut self, _model: &AppModel, _sender: Sender<AppMsg>) {}

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.window.clone()
    }
}

pub fn thread() {
    let model = AppModel {};
    let app = RelmApp::new(model);

    relm4::set_global_css(include_bytes!("main.css"));

    app.run();
}
