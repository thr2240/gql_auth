mod sidecar_plugin;

use anyhow::Result;

// `cargo run -- -s ./supergraph-schema.graphql -c ./router.yaml`
fn main() -> Result<()> {
    apollo_router::main()
}
