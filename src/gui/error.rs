use crate::Error;
use gdk::{
    gio::{SimpleAction, SimpleActionGroup},
    glib::clone,
    prelude::{ActionMapExt, Cast},
};
use gettextrs::gettext;
use gtk::{
    traits::{DialogExt, GtkWindowExt, WidgetExt},
    ButtonsType, MessageType, Window,
};
use libadwaita::{Toast, ToastOverlay};

pub fn error_to_toast(overlay: &ToastOverlay, err: Error) {
    log::error!("Displaying error: {}", err);
    let toast = match &err {
        Error::Hafas(hafas_rs::Error::Http { .. }) => Toast::new(&gettext(
            "Failed to fetch data. Are you connected to the internet?",
        )),
        Error::Hafas(hafas_rs::Error::Hafas { .. }) => Toast::builder()
            .title(gettext("The API returned a error. Please report this."))
            .button_label(gettext("More information"))
            .build(),
        Error::Hafas(hafas_rs::Error::Json { .. }) => Toast::builder()
            .title(gettext("Failed to parse response. Please report this."))
            .button_label(gettext("More information"))
            .build(),
        _ => Toast::builder()
            .title(gettext("Unknown issue. Please report this."))
            .button_label(gettext("More information"))
            .build(),
    };

    let msg = match err {
        Error::Hafas(hafas_rs::Error::Http { .. }) => None,
        Error::Hafas(hafas_rs::Error::Hafas { text, .. }) => Some(text),
        Error::Hafas(hafas_rs::Error::Json { source, .. }) => Some(format!("{}", source)),
        _ => Some(format!("{}", err)),
    };

    if let Some(msg) = msg {
        let action_more_info = SimpleAction::new("more-info", None);
        action_more_info.connect_activate(clone!(@strong overlay => move |_, _| {
            let dialog = gtk::MessageDialog::builder()
                .message_type(MessageType::Error)
                .use_header_bar(1)
                .transient_for(
                    &overlay
                        .root()
                        .expect("Overlay to have a root.")
                        .downcast::<Window>()
                        .expect("Root of overlay to be a Window."),
                )
                .deletable(true)
                .buttons(ButtonsType::Close)
                .title(gettext("Error"))
                .text(&msg)
                .build();
            dialog.present();
            dialog.connect_response(|dialog, _| dialog.close());
        }));
        toast.set_action_name(Some("toast.more-info"));

        let actions = SimpleActionGroup::new();
        overlay.insert_action_group("toast", Some(&actions));
        actions.add_action(&action_more_info);
    }

    overlay.add_toast(toast);
}
