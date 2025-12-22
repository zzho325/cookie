use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();

    // Add strum derives to the proto enums so we can iterate over all variants
    config.type_attribute("OpenAIModel", "#[derive(strum::EnumIter)]");
    config.type_attribute("GeminiModel", "#[derive(strum::EnumIter)]");

    config.compile_protos(
        &["src/proto/chat.proto", "src/proto/llm.proto"],
        &["src/proto"],
    )?;
    Ok(())
}
