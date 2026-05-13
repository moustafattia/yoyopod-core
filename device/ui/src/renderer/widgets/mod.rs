pub mod facade;
#[cfg(feature = "native-lvgl")]
pub(crate) mod factory;
pub mod primitives;
#[cfg(feature = "native-lvgl")]
pub(crate) mod registry;

pub use facade::LvglFacade;
pub use primitives::WidgetId;
