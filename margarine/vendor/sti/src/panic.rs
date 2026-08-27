
#[macro_export]
macro_rules! define_panic_handler {
    () => {
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::os::abort();
        }
    };
}

