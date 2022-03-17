use gdk::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::glib::IsA;
use gtk::traits::{GtkWindowExt, WidgetExt};

mod gui;

fn init_resources() {
    let res_bytes = include_bytes!("../resources.gresource");

    let gbytes = gtk::glib::Bytes::from_static(res_bytes.as_ref());
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_icons<P: IsA<gdk::Display>>(display: &P) {
    let icon_theme = gtk::IconTheme::for_display(display);

    icon_theme.add_resource_path("/");
}

fn init_internationalization() -> Result<(), Box<dyn std::error::Error>> {
    gettextrs::TextDomain::new("de.schmidhuberj.DieBahn")
        .locale_category(gettextrs::LocaleCategory::LcAll)
        .prepend("./po")
        .init()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    init_internationalization().expect("Failed to initialize internationalization");

    env_logger::init();
    gtk::init().expect("Failed to initialize gtk");
    libadwaita::init();
    let app = gtk::Application::builder()
        .application_id("de.schmidhuberj.DieBahn")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    init_resources();
    let window = crate::gui::window::Window::new(app);
    init_icons(&window.display());
    window.present();
}
