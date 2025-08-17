use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["src/proto/chat.proto", "src/proto/llm.proto"],
        &["src/proto"],
    )?;
    Ok(())
}
