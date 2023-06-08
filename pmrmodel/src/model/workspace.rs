use pmrmodel_base::workspace::{
    WorkspaceRecord,
    WorkspaceRecords,
};
use std::io::Write;

pub fn stream_workspace_records_default(
    mut writer: impl Write,
    records: Vec<WorkspaceRecord>,
) -> std::result::Result<usize, std::io::Error> {
    let mut result: usize = 0;
    result += writer.write(b"id - url - description\n")?;
    for record in records {
        result += writer.write(format!("{}\n", record).as_bytes())?;
    }
    Ok(result)
}

pub fn stream_workspace_records_as_json(
    writer: impl Write,
    records: Vec<WorkspaceRecord>,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, &WorkspaceRecords { workspaces: records })
}
