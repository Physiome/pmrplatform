use pmrmodel_base::workspace::{
    Workspace,
    Workspaces,
};
use std::io::Write;

pub fn stream_workspace_records_default(
    mut writer: impl Write,
    records: Vec<Workspace>,
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
    records: Vec<Workspace>,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, &Workspaces { workspaces: records })
}
