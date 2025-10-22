// Stack not subclassable, therefore have the stack as a child of the widget.
gtk::glib::wrapper! {
    pub struct RefreshButton(ObjectSubclass<imp::RefreshButton>)
        @extends libadwaita::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::marker::PhantomData;

    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::subclass::Signal;
    use gdk::glib::Properties;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::RefreshButton)]
    #[template(resource = "/ui/refresh_button.ui")]
    pub struct RefreshButton {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        page_button: TemplateChild<gtk::StackPage>,
        #[template_child]
        page_spinner: TemplateChild<gtk::StackPage>,

        #[property(set = Self::set_refreshing)]
        refreshing: PhantomData<bool>,
    }

    #[template_callbacks]
    impl RefreshButton {
        #[template_callback]
        fn handle_refresh_clicked(&self) {
            self.obj().emit_by_name::<()>("clicked", &[]);
        }

        fn set_refreshing(&self, refreshing: bool) {
            if refreshing {
                self.stack.set_visible_child(&self.page_spinner.child());
            } else {
                self.stack.set_visible_child(&self.page_button.child());
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RefreshButton {
        const NAME: &'static str = "DBRefreshButton";
        type Type = super::RefreshButton;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RefreshButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.stack.set_visible_child(&self.page_button.child());
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("clicked").build()]);
            SIGNALS.as_ref()
        }
    }

    impl BinImpl for RefreshButton {}
    impl WidgetImpl for RefreshButton {}
}
