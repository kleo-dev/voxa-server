#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        static PLUGIN: std::sync::Mutex<Option<$plugin_type>> = std::sync::Mutex::new(None);

        #[unsafe(no_mangle)]
        pub extern "C" fn init() {
            let mut plugin = PLUGIN.lock().unwrap();
            *plugin = Some(<$plugin_type>::default());
            if let Some(plugin) = plugin.as_mut() {
                plugin.init();
            }
        }
    };
}

#[macro_export]
macro_rules! logger {
    (const $i:ident $name:expr) => {
        const $i: once_cell::sync::Lazy<$crate::logger::Logger> =
            once_cell::sync::Lazy::new(|| $crate::logger::Logger::new($name));
    };

    ($i:ident $name:expr) => {
        const $i: once_cell::sync::Lazy<$crate::logger::Logger> =
            once_cell::sync::Lazy::new(|| $crate::logger::Logger::new($name));
    };

    (const $name:expr) => {
        once_cell::sync::Lazy::new(|| $crate::logger::Logger::new($name))
    };

    ($name:expr) => {
        $crate::logger::Logger::new($name)
    };
}
