pub trait Plugin {
    fn init(&mut self);
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        static mut PLUGIN: Option<$plugin_type> = None;

        #[no_mangle]
        pub extern "C" fn init() {
            unsafe {
                PLUGIN = Some(<$plugin_type>::default());
                if let Some(plugin) = PLUGIN.as_mut() {
                    plugin.init();
                }
            }
        }
    };
}
