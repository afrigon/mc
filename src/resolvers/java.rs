use crate::context::McContext;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::VersionResolver;

pub struct JavaVersionResolver;

impl VersionResolver<JavaVendor, JavaVersion> for JavaVersionResolver {
    async fn resolve(_: &McContext, version: Option<&str>) -> McResult<JavaVersion> {
        version.unwrap_or("25").parse()
    }
}
