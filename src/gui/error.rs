use crate::Error;
use gdk::{
    gio::{SimpleAction, SimpleActionGroup},
    glib::clone,
    prelude::{ActionMapExt, Cast},
};
use gettextrs::gettext;
use gtk::{
    prelude::WidgetExt,
    Window,
};
use libadwaita::{Toast, ToastOverlay, AlertDialog, prelude::AlertDialogExt, prelude::AdwDialogExt};

pub fn error_to_toast(overlay: &ToastOverlay, err: Error) {
    log::error!("Displaying error: {}", err);
    let toast = match &err {
        Error::Hafas(hafas_rs::Error::Http { .. }) => Toast::new(&gettext(
            "Failed to fetch data. Are you connected to the internet?",
        )),
        Error::Hafas(hafas_rs::Error::Hafas { .. }) => Toast::builder()
            .title(gettext("Received an error. Please share feedback."))
            .button_label(gettext("More Information"))
            .build(),
        Error::Hafas(hafas_rs::Error::Json { .. }) => Toast::builder()
            .title(gettext("Received an invalid response. Please share feedback."))
            .button_label(gettext("More Information"))
            .build(),
        _ => Toast::builder()
            .title(gettext("An unknown issue occured. Please share feedback."))
            .button_label(gettext("More Information"))
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
            let dialog = AlertDialog::builder()
                .heading(gettext("Error"))
                .body(&msg)
                .default_response("close")
                .build();
            dialog.add_response("close", &gettextrs::gettext("_Close"));
            dialog.present(&overlay
                            .root()
                            .expect("Overlay to have a root.")
                            .downcast::<Window>()
                            .expect("Root of overlay to be a Window."));
        }));
        toast.set_action_name(Some("toast.more-info"));

        let actions = SimpleActionGroup::new();
        overlay.insert_action_group("toast", Some(&actions));
        actions.add_action(&action_more_info);
    }

    overlay.add_toast(toast);
}
