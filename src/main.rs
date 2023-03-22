use gdk::prelude::{ApplicationExt, ApplicationExtManual};
use gdk::Display;
use gtk::glib::IsA;
use gtk::traits::{GtkWindowExt, WidgetExt};
use gtk::{CssProvider, StyleContext};

#[macro_export]
macro_rules! gspawn {
    ($future:expr) => {
        let ctx = gtk::glib::MainContext::default();
        ctx.spawn_local($future);
    };
}

mod backend;
mod config;
mod error;
mod gui;

use config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_BYTES};
pub use error::Error;

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/de/schmidhuberj/DieBahn/style.css");

    // Add the provider to the default screen
    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn init_resources() {
    let gbytes = gtk::glib::Bytes::from_static(RESOURCES_BYTES);
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_icons<P: IsA<gdk::Display>>(display: &P) {
    let icon_theme = gtk::IconTheme::for_display(display);

    icon_theme.add_resource_path("/");
}

fn init_internationalization() -> Result<(), Box<dyn std::error::Error>> {
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)?;
    gettextrs::textdomain(GETTEXT_PACKAGE)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    init_internationalization().expect("Failed to initialize internationalization");

    env_logger::init();
    gtk::init().expect("Failed to initialize gtk");
    libadwaita::init();
    let app = libadwaita::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &libadwaita::Application) {
    init_resources();
    load_css();
    let window = crate::gui::window::Window::new(app);
    init_icons(&window.display());
    window.present();
}
