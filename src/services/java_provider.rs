use crate::env::Architecture;
use crate::env::Platform;
use crate::java::JavaVersion;
use crate::network::artifact::ArtifactSource;

pub trait JavaProvider {
    fn jdk_source(
        java_version: JavaVersion,
        platform: Platform,
        architecture: Architecture
    ) -> ArtifactSource;
}
