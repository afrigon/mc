use crate::context::McContext;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::VersionResolver;

pub struct JavaVersionResolver;

impl VersionResolver<JavaVendor, JavaVersion> for JavaVersionResolver {
    async fn resolve(context: &McContext, version: Option<String>) -> McResult<JavaVersion> {
        version.unwrap_or_else(|| "25".to_owned()).parse()
    }
}
