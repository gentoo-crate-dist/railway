// Stack not subclassable, therefore have the stack as a child of the widget.
gtk::glib::wrapper! {
    pub struct RefreshButton(ObjectSubclass<imp::RefreshButton>)
        @extends libadwaita::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::subclass::Signal;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::Value;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/refresh_button.ui")]
    pub struct RefreshButton {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        page_button: TemplateChild<gtk::StackPage>,
        #[template_child]
        page_spinner: TemplateChild<gtk::StackPage>,
    }

    #[template_callbacks]
    impl RefreshButton {
        #[template_callback]
        fn handle_refresh_clicked(&self) {
            self.obj().emit_by_name::<()>("clicked", &[]);
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

    impl ObjectImpl for RefreshButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.stack.set_visible_child(&self.page_button.child());
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecBoolean::builder("refreshing").write_only().build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "refreshing" => {
                    let refreshing = value.get::<bool>().expect(
                        "Property `refreshing` of `RefreshButton` has to be of type `bool`",
                    );

                    if refreshing {
                        self.stack.set_visible_child(&self.page_spinner.child());
                    } else {
                        self.stack.set_visible_child(&self.page_button.child());
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
            unimplemented!()
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
