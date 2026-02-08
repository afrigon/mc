use crate::context::McContext;
use crate::mods::loader::LoaderKind;
use crate::resolvers::fabric::FabricVersionResolver;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

pub struct LoaderVersionResolver;

impl VersionResolver<LoaderKind> for LoaderVersionResolver {
    async fn resolve(context: &McContext, version: Option<String>) -> McResult<String> {
        Err(utils::errors::internal(
            "LoaderVersionResolver::resolve() cannot be called explicitly"
        ))
    }

    async fn resolve_descriptor(
        context: &McContext,
        descriptor: RawProductDescriptor
    ) -> McResult<ProductDescriptor<LoaderKind, String>> {
        match descriptor.product.as_str() {
            "fabric" => FabricVersionResolver::resolve_descriptor(context, descriptor).await,
            loader => anyhow::bail!("unknown loader {}", loader)
        }
    }
}
