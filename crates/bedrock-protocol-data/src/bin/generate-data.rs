//! Generate embedded protocol version data from protocol-docs directory.
//!
//! Usage:
//! ```bash
//! cargo run -p bedrock-protocol-data -- generate-data \
//!   --docs ./docs/protocol-docs \
//!   --output ./data/v975.json
//! ```
//!
//! This reads the protocol-docs directory, creates an EmbeddedVersion,
//! and serializes it to a JSON file that can be compiled into the binary.

use bedrock_protocol_schema::embed::EmbeddedVersion;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let docs_path = parse_arg(&args, "--docs")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default: workspace-root-relative
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop(); p.pop(); p.pop(); // go from crate to workspace
            p.join("docs").join("protocol-docs")
        });

    let output_path = parse_arg(&args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.push("data");
            // Derive filename from version
            let readme = std::fs::read_to_string(docs_path.join("README.md"))
                .unwrap_or_default();
            let version = readme.lines()
                .find(|l| l.contains("Network Version"))
                .and_then(|l| l.split(':').last())
                .map(|s| s.trim().trim_matches('*').trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            p.push(format!("v{}.json", version));
            p
        });

    println!("Reading protocol docs from: {}", docs_path.display());
    println!("Output: {}", output_path.display());

    let embedded = EmbeddedVersion::from_directory(&docs_path)
        .expect("Failed to load protocol docs");

    let json = serde_json::to_string_pretty(&embedded)
        .expect("Failed to serialize embedded version");

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }
    std::fs::write(&output_path, &json)
        .expect("Failed to write output file");

    println!("Done. Generated {} bytes for protocol v{} ({}).",
        json.len(),
        embedded.network_version,
        embedded.branch_name);

    println!("\nSummary:");
    println!("  Network Version: {}", embedded.network_version);
    println!("  Branch: {}", embedded.branch_name);
    println!("  Minecraft: {}", embedded.minecraft_version);
    println!("  Packets: {}", embedded.packets.len());
    println!("  Enums: {}", embedded.enums.len());
    println!("  Types: {}", embedded.types.len());
}

fn parse_arg(args: &[String], name: &str) -> Option<String> {
    let pos = args.iter().position(|a| a == name)?;
    args.get(pos + 1).cloned()
}
