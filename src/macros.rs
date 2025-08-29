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
