use std::time::Duration;

use gdk::glib::{self, object::ObjectExt, Object};

gtk::glib::wrapper! {
    pub struct Timer(ObjectSubclass<imp::Timer>);
}

impl Default for Timer {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Timer {
    async fn run(&self) {
        loop {
            self.emit_by_name::<()>("minutely", &[]);
            glib::timeout_future(Duration::from_secs(60)).await;
        }
    }
}

mod imp {
    use gdk::glib::subclass::object::{ObjectImpl, ObjectImplExt};
    use gdk::glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
    use gdk::glib::subclass::Signal;
    use gdk::glib::{self, clone};
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct Timer {}

    #[glib::object_subclass]
    impl ObjectSubclass for Timer {
        const NAME: &'static str = "DBTimer";
        type Type = super::Timer;
    }

    impl ObjectImpl for Timer {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            gspawn!(clone!(
                #[weak]
                obj,
                async move {
                    obj.run().await;
                }
            ));
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| -> Vec<Signal> { vec![Signal::builder("minutely").build()] });
            SIGNALS.as_ref()
        }
    }
}
