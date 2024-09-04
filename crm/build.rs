use std::fs;

use proto_builder_trait::tonic::BuilderAttributes;

fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        // .with_field_attributes(
        //     &[
        //         "WelcomeRequest.id",
        //         "WelcomeRequest.interval",
        //         "WelcomeRequest.content_ids",
        //         "RecallRequest.id",
        //         "RecallRequest.last_visit_interval",
        //         "RecallRequest.last_watched_interval",
        //         "RecallRequest.content_ids",
        //         "RemindRequest.id",
        //         "RemindRequest.last_visit_interval",
        //     ],
        //     &[r#"#[builder(setter(into))]"#],
        // )
        .compile(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos"],
        )?;
    Ok(())
}
