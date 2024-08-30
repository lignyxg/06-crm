use std::fs;

fn main() -> anyhow::Result<()> {
    fs::create_dir_all("src/pb")?;

    let builder = tonic_build::configure();

    builder
        .out_dir("src/pb")
        // .with_serde(
        //     &["User"],
        //     true,
        //     true,
        //     Some(&[r#"#[serde(rename_all = "camelCase")]"#]),
        // )
        // .with_sqlx_from_row(&["User"], None)
        // .with_derive_builder(
        //     &[
        //         "User",
        //         "QueryRequest",
        //         "TimeQuery",
        //         "IdQuery",
        //         "RawQueryRequest",
        //     ],
        //     None,
        // )
        // .with_field_attributes(
        //     &["QueryRequest.timestamps"],
        //     &[r#"#[builder(setter(each(name = "timestamp_builder",into)))]"#],
        // )
        // .with_field_attributes(
        //     &["QueryRequest.ids"],
        //     &[r#"#[builder(setter(each(name = "id_builder",into)))]"#],
        // )
        // .with_field_attributes(
        //     &["User.email", "User.name", "RawQueryRequest.query"],
        //     &[r#"#[builder(setter(into))]"#],
        // )
        // .with_field_attributes(
        //     &["TimeQuery.lower", "TimeQuery.upper"],
        //     &[r#"#[builder(setter(into, strip_option))]"#],
        // )
        .compile(
            &[
                "../protos/metadata/messages.proto",
                "../protos/metadata/rpc.proto",
            ],
            &["../protos"],
        )?;

    Ok(())
}
