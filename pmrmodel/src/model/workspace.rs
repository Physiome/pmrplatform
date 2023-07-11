use pmrmodel_base::workspace::Workspaces;
use std::io::Write;

pub fn stream_workspace_records_default(
    mut writer: impl Write,
    workspaces: &Workspaces,
) -> std::result::Result<usize, std::io::Error> {
    let mut result: usize = 0;
    result += writer.write(b"id - url - description\n")?;
    for workspace in workspaces.iter() {
        result += writer.write(format!("{}\n", workspace).as_bytes())?;
    }
    Ok(result)
}

pub fn stream_workspace_records_as_json(
    writer: impl Write,
    workspaces: &Workspaces,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, workspaces)
}
