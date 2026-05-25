fn main() {
    // ── Code Generation ────────────────────────────────────────────────
    // Reads compile-time embedded protocol data (committed to git) and
    // generates Rust code for 190+ packets, 108 enums, 150+ types.
    //
    // To update protocol data for a new version:
    //   cargo run -p bedrock-protocol-data --bin generate-data -- \
    //     --docs ./docs/protocol-docs \
    //     --output ./crates/bedrock-protocol-data/data/v{N}.json
    // Then register it in bedrock-protocol-data/src/lib.rs.

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let reg = bedrock_protocol_data::registry();

    // Default to v975 (current stable). Override with PROTOCOL_VERSION env var.
    let version: u32 = std::env::var("PROTOCOL_VERSION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(975);

    let schema = reg.get(version).unwrap_or_else(|| {
        panic!("bedrock-protocol build: protocol version {} not found in embedded data", version)
    });
    let pcount = schema.packet_count();
    let ecount = schema.enum_count();
    let tcount = schema.type_count();

    if let Err(e) = bedrock_codegen::generate_all(schema, &out_dir) {
        eprintln!("bedrock-protocol build: code generation failed: {}", e);
        return;
    }

    println!("cargo:info=Generated protocol code ({} packets, {} enums, {} types)", pcount, ecount, tcount);
}
