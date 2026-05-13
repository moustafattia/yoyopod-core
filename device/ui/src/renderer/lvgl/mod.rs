#[cfg(feature = "native-lvgl")]
pub(crate) mod backend;
#[cfg(feature = "native-lvgl")]
pub(crate) mod ffi;
#[cfg(feature = "native-lvgl")]
pub(crate) mod flush;
#[cfg(feature = "native-lvgl")]
pub(crate) mod icons;
#[cfg(feature = "native-lvgl")]
pub(crate) mod lifecycle;
#[cfg(feature = "native-lvgl")]
pub(crate) mod node_registry;
pub mod renderer;

#[cfg(feature = "native-lvgl")]
pub use backend::NativeLvglFacade;
pub use renderer::LvglRenderer;
