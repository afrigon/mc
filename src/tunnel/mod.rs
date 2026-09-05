mod provider;

pub use provider::TunnelProviderKind;

use crate::utils::product_descriptor::ProductDescriptor;

pub type TunnelDescriptor = ProductDescriptor<TunnelProviderKind, String>;
