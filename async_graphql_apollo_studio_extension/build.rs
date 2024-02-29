// Derived from https://github.com/pellizzetti/router/blob/cc0ebcaf1d68184e1fe06f16534fddff76286b40/apollo-spaceport/build.rs
use std::{
    error::Error,
    fs::File,
    io::{copy, Read},
};

fn main() -> Result<(), Box<dyn Error>> {
    // Skip building from docs.rs.
    if std::env::var_os("DOCS_RS").is_some() {
        return Ok(());
    }
    // Retrieve a live version of the reports.proto file
    let proto_url = "https://usage-reporting.api.apollographql.com/proto/reports.proto";
    let response = reqwest::blocking::get(proto_url)?;
    let mut content = response.text()?;

    // Process the retrieved content to:
    //  - Insert a package Report; line after the import lines (currently only one) and before the first message definition
    //  - Remove the Apollo TS extensions [(js_use_toArray)=true] and [(js_preEncoded)=true] from the file
    //  Note: Only two in use at the moment. This may fail in future if new extensions are
    //  added to the source, so be aware future self. It will manifest as a protobuf compile
    //  error.
    let message = "\nmessage";
    let msg_index = content.find(message).ok_or("cannot find message string")?;
    content.insert_str(msg_index, "\npackage Report;\n");

    content = content.replace("[(js_use_toArray) = true]", "");
    content = content.replace("[(js_preEncoded) = true]", "");

    // Try to avoid writing out the same content since it will trigger unnecessary re-builds, which wastes time
    let write_content = match File::open("proto/reports.proto") {
        Ok(mut existing) => {
            let mut existing_content = String::new();
            existing.read_to_string(&mut existing_content)?;
            content != existing_content
        }
        Err(_) => true,
    };

    // Write the content out if they differ or an error occured trying to open proto file
    if write_content {
        let mut dest = File::create("proto/reports.proto")?;
        copy(&mut content.as_bytes(), &mut dest)?;
    }

    // Process the proto files
    let proto_files = vec!["proto/agents.proto", "proto/reports.proto"];

    tonic_build::configure()
        .type_attribute("ContextualizedStats", "#[derive(serde::Serialize)]")
        .type_attribute("StatsContext", "#[derive(serde::Serialize)]")
        .type_attribute("QueryLatencyStats", "#[derive(serde::Serialize)]")
        .type_attribute("TypeStat", "#[derive(serde::Serialize)]")
        .type_attribute("PathErrorStats", "#[derive(serde::Serialize)]")
        .type_attribute("FieldStat", "#[derive(serde::Serialize)]")
        .type_attribute("ReferencedFieldsForType", "#[derive(serde::Serialize)]")
        .type_attribute("StatsContext", "#[derive(Eq, Hash)]")
        .build_server(true)
        .compile(&proto_files, &["."])?;

    for file in proto_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    Ok(())
}
