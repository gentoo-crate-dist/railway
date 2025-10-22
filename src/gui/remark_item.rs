use gdk::glib::Object;

use crate::backend::Remark;

gtk::glib::wrapper! {
    pub struct RemarkItem(ObjectSubclass<imp::RemarkItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl RemarkItem {
    pub fn new(remark: &Remark) -> Self {
        Object::builder::<Self>().property("remark", remark).build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::DirectionType;

    use crate::backend::Remark;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::RemarkItem)]
    #[template(resource = "/ui/remark_item.ui")]
    pub struct RemarkItem {
        #[property(get, set)]
        remark: RefCell<Option<Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RemarkItem {
        const NAME: &'static str = "DBRemarkItem";
        type Type = super::RemarkItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            WidgetClassExt::set_css_name(klass, "AnnouncementItem");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RemarkItem {}

    impl WidgetImpl for RemarkItem {
        fn focus(&self, direction: DirectionType) -> bool {
            Utility::move_focus_within_container(self, direction)
        }
    }

    impl BoxImpl for RemarkItem {}
}
