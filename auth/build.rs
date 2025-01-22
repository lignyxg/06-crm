use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_wkt_types")
        .with_serde(
            &[
                "SignRequest",
                "VerifyRequest",
                "SignResponse",
                "VerifyResponse",
            ],
            true,
            true,
            None,
        )
        .with_derive_builder(&["SignRequest", "VerifyRequest"], None)
        .compile(
            &["../protos/auth/messages.proto", "../protos/auth/rpc.proto"],
            &["../protos"],
        )?;

    Ok(())
}
