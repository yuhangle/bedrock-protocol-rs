//! Protocol version registry — supports compile-time embedded protocol data
//! and runtime multi-version queries (ViaVersion-style).
//!
//! # Architecture
//!
//! Each protocol version's JSON data is serialized to a compact JSON blob
//! at build time and embedded via `include_str!()`. At runtime, the registry
//! deserializes and indexes the data for fast lookup by protocol version number.

use crate::{embed::EmbeddedVersion, Schema};
use std::collections::HashMap;

/// Thread-safe registry of protocol versions.
///
/// Built from compile-time embedded data. Each version is keyed by its
/// network protocol version number (e.g., 975 for r26_u2).
pub struct ProtocolRegistry {
    versions: HashMap<u32, Schema>,
    latest_version: u32,
    version_meta: HashMap<u32, VersionMeta>,
}

/// Metadata about a protocol version.
#[derive(Debug, Clone)]
pub struct VersionMeta {
    pub network_version: u32,
    pub branch_name: String,
    pub minecraft_version: String,
    pub packet_count: usize,
}

impl ProtocolRegistry {
    /// Build a registry from a list of embedded version data blobs.
    ///
    /// Each blob is a JSON-serialized `EmbeddedVersion` (from `include_str!()`).
    pub fn from_embedded(version_blobs: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut versions = HashMap::new();
        let mut version_meta = HashMap::new();
        let mut max_version = 0u32;

        for blob in version_blobs {
            let embedded: EmbeddedVersion = serde_json::from_str(blob)?;
            let nv = embedded.network_version;
            let pcount = embedded.packets.len();
            let schema = embedded.to_schema()?;

            version_meta.insert(nv, VersionMeta {
                network_version: nv,
                branch_name: embedded.branch_name.clone(),
                minecraft_version: embedded.minecraft_version.clone(),
                packet_count: pcount,
            });

            versions.insert(nv, schema);
            if nv > max_version {
                max_version = nv;
            }
        }

        Ok(Self {
            versions,
            latest_version: max_version,
            version_meta,
        })
    }

    /// Get the schema for a specific protocol version.
    pub fn get(&self, network_version: u32) -> Option<&Schema> {
        self.versions.get(&network_version)
    }

    /// Get the latest available protocol version.
    pub fn latest(&self) -> &Schema {
        self.versions
            .get(&self.latest_version)
            .expect("ProtocolRegistry: no versions loaded")
    }

    /// Get the latest version number.
    pub fn latest_version(&self) -> u32 {
        self.latest_version
    }

    /// Get metadata for all available versions.
    pub fn all_versions(&self) -> impl Iterator<Item = &VersionMeta> {
        self.version_meta.values()
    }

    /// Check if a specific version is available.
    pub fn has_version(&self, network_version: u32) -> bool {
        self.versions.contains_key(&network_version)
    }

    /// Number of available versions.
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Macro to build a ProtocolRegistry from compile-time embedded data.
///
/// Usage in build.rs: generate modules like `protocol_v975`, `protocol_v1001`,
/// each containing `pub const DATA: &str = ...;`.
///
/// Then in lib.rs:
/// ```ignore
/// ProtocolRegistry::from_embedded(&[
///     include_str!(concat!(env!("OUT_DIR"), "/protocol_v975.json")),
///     include_str!(concat!(env!("OUT_DIR"), "/protocol_v1001.json")),
/// ])
/// ```
/// Build a ProtocolRegistry from compile-time embedded version data.
///
/// Each argument is a module name that contains `pub const DATA: &str`
/// pointing to a serialized `EmbeddedVersion`.
///
/// ```ignore
/// build_registry!(v975, v1001)
/// ```
#[macro_export]
macro_rules! build_registry {
    ($($mod:ident),* $(,)?) => {{
        let blobs: &[&str] = &[$(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"), "/data/", stringify!($mod), ".json"
        ))),*];
        $crate::registry::ProtocolRegistry::from_embedded(blobs)
            .expect("Failed to build protocol registry")
    }};
}
