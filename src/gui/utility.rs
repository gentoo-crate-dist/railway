use gdk::glib::Object;

pub struct Utility {}

#[gtk::template_callbacks(functions)]
impl Utility {
    #[template_callback]
    fn and(#[rest] values: &[gtk::glib::Value]) -> bool {
        let val0 = values[0]
            .get::<bool>()
            .expect("Expected boolean for argument");
        let val1 = values[1]
            .get::<bool>()
            .expect("Expected boolean for argument");
        val0 && val1
    }

    #[template_callback]
    fn is_some(#[rest] values: &[gtk::glib::Value]) -> bool {
        values
            .iter()
            .next()
            .expect("At least one argument has to exist")
            .get::<Option<Object>>()
            .expect("Expected Option for arguments")
            .is_some()
    }

    #[template_callback]
    fn is_none(#[rest] values: &[gtk::glib::Value]) -> bool {
        !Utility::is_some(values)
    }

    #[template_callback]
    fn not(value: bool) -> bool {
        !value
    }
}
