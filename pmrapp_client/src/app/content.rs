use crate::app;
use sauron::prelude::*;
use serde::{Deserialize, Serialize};

use pmrmodel_base::{
    workspace::{
        Workspaces,
        Workspace,
    },
    repo::{
        PathInfo,
        PathObject,
        TreeEntryInfo,
    },
    merged::{
        WorkspacePathInfo,
    }
};

use crate::model::JsonWorkspaceRecord;
use crate::app::Resource;
use crate::app::Msg;

#[derive(
    Debug, Deserialize, Serialize, PartialEq, Clone, derive_more::From,
)]
pub enum Content {
    Homepage,
    WorkspaceListing(Workspaces),
    WorkspaceTop(JsonWorkspaceRecord, Option<WorkspacePathInfo>),
    WorkspacePathInfo(WorkspacePathInfo),
}


impl Content {
    pub fn view(&self) -> Node<app::Msg> {
        match self {
            Content::Homepage => {
                node! {
                    <div class="main">
                        <h1>"Physiome Model Repository"</h1>
                        <p>
                          "Welcome to the demo of the platform that will \n\
                          power the next generation of the Physiome Model \n\
                          Repository, written in Rust."
                        </p>
                        <p>
                          "The code for this project may be found on "
                          <a href="https://github.com/Physiome/pmrplatform/">
                            "its project page on GitHub"
                          </a>
                          "."
                        </p>
                        <dl>
                            <dt><a href="/workspace/"
                                on_click=|e| {
                                    e.prevent_default();
                                    Msg::Retrieve(Resource::WorkspaceListing, "/workspace/".to_string())
                                }>"Workspace Listing"</a></dt>
                          <dd>"Listing of all workspaces in the repository."</dd>
                        </dl>
                    </div>
                }
            }
            Content::WorkspaceListing(workspaces) => {
                node! {
                    <div class="main">
                        <h1>"Workspace Listing"</h1>
                        <div class="workspace-listing">
                        {
                            for workspace in workspaces.iter() {
                                self.show_workspace(workspace)
                            }
                        }
                        </div>
                    </div>
                }
            },
            Content::WorkspaceTop(entry, wks_path_info) => {
                node! {
                    <div class="main">
                        <h1>{ text!("{}", &entry.workspace.description.as_ref().unwrap_or(
                            &format!("Workspace {}", &entry.workspace.id))) }</h1>
                        <dl>
                            <dt>"Git Repository URI"</dt>
                            <dd>{ text!("{}", &entry.workspace.url) }</dd>
                        </dl>
                        <div class="workspace-pathinfo">
                        {
                            self.show_workspace_file_table((&wks_path_info).as_ref())
                        }
                        </div>
                    </div>
                }
            },
            Content::WorkspacePathInfo(wks_path_info) => {
                let workspace_id = wks_path_info.workspace_id;
                node! {
                    <div class="main">
                        <h1>
                            <a
                                relative
                                href=format!("/workspace/{}/", &wks_path_info.workspace_id)
                                on_click=move |e| {
                                    e.prevent_default();
                                    Msg::Retrieve(
                                        Resource::WorkspaceTop(workspace_id),
                                        format!("/workspace/{}/", workspace_id)
                                    )
                                }>
                            {
                                text!("{}", &wks_path_info.description.as_ref().unwrap_or(
                                    &format!("Workspace {}", &wks_path_info.workspace_id)))
                            }
                            </a>
                        </h1>
                        <div class="workspace-pathinfo">
                        {
                            match &wks_path_info.object {
                                Some(PathObject::TreeInfo(..)) => {
                                    self.show_workspace_file_table(Some(&wks_path_info))
                                }
                                Some(PathObject::FileInfo(file_info)) => {
                                    let href = format!(
                                        "/workspace/{}/raw/{}/{}",
                                        &wks_path_info.workspace_id,
                                        &wks_path_info.commit.commit_id,
                                        &wks_path_info.path,
                                    );
                                    node! {
                                        <div>
                                        {
                                            text!("{:?}", file_info)
                                        }
                                        </div>
                                        <div>
                                            <a href=&href>"download"</a>
                                        </div>
                                        {
                                            if &file_info.mime_type[..5] == "image" {
                                                node! {
                                                    <div>
                                                        <p>"Preview"</p>
                                                        <img src=&href />
                                                    </div>
                                                }
                                            }
                                            else {
                                                node! {
                                                    <div></div>
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    text!("")
                                }
                            }
                        }
                        </div>
                    </div>
                }
            },
        }
    }

    fn show_workspace(&self, workspace: &Workspace) -> Node<app::Msg> {
        let workspace_id = workspace.id;
        node! {
            <div>
                <div><a
                    relative
                    href=format!("/workspace/{}/", workspace_id)
                    on_click=move |e| {
                        e.prevent_default();
                        Msg::Retrieve(Resource::WorkspaceTop(workspace_id), format!("/workspace/{}/", workspace_id))
                    }
                >{ text!("Workspace: {}", workspace_id) }
                </a></div>
                <div>{ text!("{}", workspace.url) }</div>
                <div>{ text!("{}", workspace.description.as_ref().unwrap_or(&"".to_string())) }</div>
            </div>
        }
    }

    fn show_workspace_file_table(&self, wks_path_info: Option<&WorkspacePathInfo>) -> Node<app::Msg> {
        node! {
            <table class="file-listing">
                <thead>
                    <tr>
                        <th>"Filename"</th>
                        <th>"Size"</th>
                        <th>"Date"</th>
                    </tr>
                </thead>
                {
                    self.show_workspace_file_table_body(wks_path_info)
                }
            </table>
        }
    }

    fn show_workspace_file_table_body(&self, wks_path_info: Option<&WorkspacePathInfo>) -> Node<app::Msg> {
        match wks_path_info {
            Some(wks_path_info) => {
                match &wks_path_info.object {
                    Some(PathObject::TreeInfo(tree_info)) => {
                        node! {
                            <tbody>
                            {
                                if wks_path_info.path != "" {
                                    self.show_workspace_file_row(
                                        wks_path_info.workspace_id,
                                        wks_path_info.commit.commit_id.clone(),
                                        wks_path_info.path.clone(),
                                        "pardir",
                                        "..",
                                    )
                                }
                                else {
                                    node! {}
                                }
                            }
                            {
                                for info in tree_info.entries.iter() {
                                    self.show_workspace_file_row(
                                        wks_path_info.workspace_id,
                                        wks_path_info.commit.commit_id.clone(),
                                        wks_path_info.path.clone(),
                                        &info.kind,
                                        &info.name,
                                    )
                                }
                            }
                            </tbody>
                        }
                    },
                    _ => node! {},
                }
            }
            None => node! {},
        }
    }

    fn show_workspace_file_row(
        &self,
        workspace_id: i64,
        commit_id: String,
        path: String,
        kind: &str,
        name: &str,
    ) -> Node<app::Msg> {
        let path_name = if name == ".." {
            let idx = path[0..path.len() - 1].rfind('/').unwrap_or(0);
            if idx == 0 {
                "".to_string()
            } else {
                format!("{}/", &path[0..idx])
            }
        } else {
            format!("{}{}", path, if kind == "tree" {
                format!("{}/", name)
            } else {
                format!("{}", name)
            })
        };
        let href = format!("/workspace/{}/file/{}/{}", workspace_id, commit_id, &path_name);
        // Sauron needs this key attribute, otherwise the correct event
        // sometimes don't get patched in...
        // https://github.com/ivanceras/sauron/issues/63
        let key = path_name.clone();
        // TODO need to test putting a proper key at the table itself...
        node! {
            <tr key=key>
                <td class=format!("gitobj-{}", kind)><span><a
                    href=&href
                    on_click=move |e| {
                        e.prevent_default();
                        Msg::Retrieve(
                            Resource::WorkspacePathInfo(
                                workspace_id,
                                commit_id.clone(),
                                path_name.clone(),
                            ),
                            href.clone(),
                        )
                    }
                    >{ text!("{}", name) }</a></span>
                </td>
                <td></td>
                <td></td>
            </tr>
        }
    }

}
