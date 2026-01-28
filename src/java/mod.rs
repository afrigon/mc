mod vendor;
mod version;

pub use vendor::JavaVendor;
pub use version::JavaVersion;

use crate::utils::CaseIterable;
use crate::utils::product_descriptor::ProductDescriptor;

pub type JavaDescriptor = ProductDescriptor<JavaVendor, JavaVersion>;

impl CaseIterable for ProductDescriptor<JavaVendor, JavaVersion> {
    fn all_cases() -> &'static [Self] {
        &[
            ProductDescriptor {
                product: JavaVendor::graal,
                version: JavaVersion::Java25
            },
            ProductDescriptor {
                product: JavaVendor::graal,
                version: JavaVersion::Java21
            },
            ProductDescriptor {
                product: JavaVendor::graal,
                version: JavaVersion::Java17
            },
            ProductDescriptor {
                product: JavaVendor::correto,
                version: JavaVersion::Java25
            },
            ProductDescriptor {
                product: JavaVendor::correto,
                version: JavaVersion::Java21
            },
            ProductDescriptor {
                product: JavaVendor::correto,
                version: JavaVersion::Java17
            },
            ProductDescriptor {
                product: JavaVendor::correto,
                version: JavaVersion::Java11
            },
            ProductDescriptor {
                product: JavaVendor::correto,
                version: JavaVersion::Java8
            }
        ]
    }
}
