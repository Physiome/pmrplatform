use pmrmodel_base::{
    workspace::{
        Workspaces,
        // Workspace,
    },
    repo::RepoResult,
};
use sauron::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;
use web_sys::PopStateEvent;

mod content;

use content::Content;
use crate::error::ServerError;
use crate::api;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchStatus<T> {
    Idle,
    Complete(T),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Resource {
    Unset,

    Homepage,
    WorkspaceListing,
    WorkspaceTop(i64),
    WorkspaceRepoResult(i64, String, String),
}

pub enum Msg {
    Retrieve(Resource, String),

    // new content and url
    ReceivedContent(Resource, Content),
    // for dealing with error responses
    RequestError(ServerError),
    // for the URL push state
    UrlChanged(Resource, String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    pub content: FetchStatus<Content>,
    is_loading: bool,
    resource: Resource,
}

impl Default for App {
    fn default() -> Self {
        Self {
            content: FetchStatus::Idle,
            is_loading: true,
            resource: Resource::Unset,
        }
    }
}

impl Application<Msg> for App {

    #[cfg(feature = "wasm")]
    fn init(&mut self) -> Cmd<Self, Msg> {
        // Only calling this init if this is not default, i.e. not the default app
        // created when loading failed in client/src/lib.rs main function
        if self.resource == Resource::Unset {
            return Cmd::none();
        }
        let mut commands = vec![];
        let listen_to_url_changes = Window::add_event_listeners(vec![on_popstate(|e| {
            log::trace!("pop_state is triggered in sauron add event listener - state: {:#?}", PopStateEvent::from(JsValue::from(&e)).state());
            let url = sauron::window()
                .location()
                .pathname()
                .expect("must have get a pathname");
            // TODO if the state is unsupported, this blows up
            // TODO rather than unwrap, if error, trigger a redirect to url
            Msg::UrlChanged(
                serde_wasm_bindgen::from_value(
                    PopStateEvent::from(JsValue::from(e)).state()
                ).unwrap(),
                url,
            )
        })]);

        let history = sauron::window().history().expect("must have history");
        log::trace!("setting initial state: {:#?}", self.resource);
        history
            .replace_state(&serde_wasm_bindgen::to_value(&self.resource).unwrap(), "")
            .expect("must push state");

        commands.push(listen_to_url_changes);
        Cmd::batch(commands)
    }

    fn view(&self) -> Node<Msg> {
        node! {
            <body class="main">
                <header>
                    <a relative href="/"
                        class={ match self.resource {
                            Resource::Homepage => "active",
                            _ => ""
                        } }
                        on_click=|e| {
                            e.prevent_default();
                            Msg::Retrieve(Resource::Homepage, "/".to_string())
                        }>
                        "Home"
                    </a>
                    <a relative href="/workspace/"
                        class={ match self.resource {
                            Resource::WorkspaceListing |
                            Resource::WorkspaceTop(..) |
                            Resource::WorkspaceRepoResult(..) => "active",
                            _ => ""
                        } }
                        on_click=|e| {
                            e.prevent_default();
                            Msg::Retrieve(Resource::WorkspaceListing, "/workspace/".to_string())
                        }>
                        "Workspace Listing"
                    </a>
                </header>
                { self.loading_indicator() }
                <main class="content">
                    { self.view_content() }
                </main>
            </body>
        }
    }

    #[cfg(not(feature = "wasm"))]
    fn update(&mut self, _msg: Msg) -> Cmd<Self, Msg> {
        Cmd::none()
    }

    #[cfg(feature = "wasm")]
    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        let mut update_resource = |resource: Resource| {
            self.is_loading = true;
            self.resource = resource.clone();
            match resource {
                Resource::Homepage => {
                    self.fetch_homepage(resource)
                }
                Resource::WorkspaceListing => {
                    self.fetch_workspace_listing(resource)
                }
                Resource::WorkspaceTop(workspace_id) => {
                    self.fetch_workspace(resource, workspace_id)
                }
                Resource::WorkspaceRepoResult(workspace_id, ref commit_id, ref path) => {
                    let commit_id = commit_id.clone();
                    let path = path.clone();
                    self.fetch_workspace_pathinfo(
                        resource,
                        workspace_id,
                        commit_id,
                        path,
                    )
                }
                Resource::Unset => {
                    Cmd::none()
                }
            }
        };

        match msg {
            // core application related
            Msg::Retrieve(resource, url) => {
                Self::push_state(&resource, &url);
                update_resource(resource)
            }

            // System related
            Msg::ReceivedContent(resource, content) => {
                if resource == self.resource {
                    self.content = FetchStatus::Complete(content);
                    self.is_loading = false;
                    log::trace!("content prepared for resource {:?}", &resource);
                    Window::scroll_to_top()
                }
                else {
                    log::warn!("fetched resource {:?} not match current resource {:?}; doing nothing", &resource, &self.resource);
                    Cmd::none()
                }
            }
            Msg::RequestError(server_error) => {
                self.is_loading = false;
                log::error!("Error: {}", server_error);
                Cmd::none()
            }
            Msg::UrlChanged(resource, url) => {
                log::trace!("UrlChanged: {}", url);
                update_resource(resource)
            }
        }
    }

    fn style(&self) -> std::string::String {
        // TODO figure out how/where should this be integrated/merged with
        // existing definitions
        "".to_string()
    }
}

impl App {
    fn view_content(&self) -> Node<Msg> {
        match &self.content {
            FetchStatus::Idle => node! { <p>"idling"</p> },
            FetchStatus::Error(e) => {
                node! {
                    <article>
                        <p>"Error: "</p>
                        <code>{text(e)}</code>
                    </article>
                }
            }
            FetchStatus::Complete(content) => content.view(),
        }
    }

    fn loading_indicator(&self) -> Node<Msg> {
        node! {
            <div id="indicator">
                <div class={ if self.is_loading {
                    "loading"
                } else {
                    "loaded"
                } }></div>
            </div>
        }
    }
}

impl App {
    pub fn with_homepage() -> Self {
        Self {
            content: FetchStatus::Complete(Content::Homepage),
            is_loading: false,
            resource: Resource::Homepage,
        }
    }

    pub fn with_workspace_listing(workspaces: Workspaces) -> Self {
        Self {
            content: FetchStatus::Complete(Content::WorkspaceListing(workspaces)),
            is_loading: false,
            resource: Resource::WorkspaceListing,
        }
    }

    pub fn with_workspace_top(
        workspace_id: i64,
        repo_result: RepoResult,
    ) -> Self {
        Self {
            content: FetchStatus::Complete(Content::WorkspaceTop(repo_result)),
            is_loading: false,
            resource: Resource::WorkspaceTop(workspace_id),
        }
    }

    pub fn with_workspace_pathinfo(
        workspace_id: i64,
        commit_id: String,
        filepath: String,
        object_info: RepoResult,
    ) -> Self {
        Self {
            content: FetchStatus::Complete(Content::WorkspaceRepoResult(object_info)),
            is_loading: false,
            resource: Resource::WorkspaceRepoResult(workspace_id, commit_id, filepath),
        }
    }
}

#[cfg(feature = "wasm")]
impl App {
    fn fetch_homepage(&self, resource: Resource) -> Cmd<Self, Msg> {
        Cmd::new(move|program| {
            program.dispatch(Msg::ReceivedContent(resource, Content::Homepage));
        })
    }

    fn fetch_workspace_listing(&self, resource: Resource) -> Cmd<Self, Msg> {
        Cmd::new(move|program| {
            let async_fetch = |program:Program<Self,Msg>| async move{
                match api::get_workspace_listing().await {
                    Ok(workspaces) => {
                        program.dispatch(Msg::ReceivedContent(resource, Content::WorkspaceListing(
                            workspaces,
                        )));
                    }
                    Err(e) => {
                        program.dispatch(Msg::RequestError(e));
                    }
                }
            };
            spawn_local(async_fetch(program))
        })
    }

    fn fetch_workspace(&self, resource: Resource, workspace_id: i64) -> Cmd<Self, Msg> {
        Cmd::new(move|program| {
            let async_fetch = |program:Program<Self,Msg>| async move{
                match api::get_workspace_top(&workspace_id).await {
                    Ok(repo_result) => {
                        program.dispatch(Msg::ReceivedContent(
                            resource,
                            Content::WorkspaceTop(repo_result),
                        ));
                    }
                    Err(e) => {
                        program.dispatch(Msg::RequestError(e));
                    }
                }
            };
            spawn_local(async_fetch(program))
        })
    }

    fn fetch_workspace_pathinfo(
        &self,
        resource: Resource,
        workspace_id: i64,
        commit_id: String,
        path: String,
    ) -> Cmd<Self, Msg> {
        Cmd::new(move|program| {
            let async_fetch = |program:Program<Self,Msg>| async move{
                match api::get_workspace_pathinfo(
                    &workspace_id,
                    &commit_id,
                    Some(&path),
                ).await {
                    Ok(repo_result) => {
                        program.dispatch(Msg::ReceivedContent(resource, Content::WorkspaceRepoResult(
                            repo_result,
                        )));
                    }
                    Err(e) => {
                        program.dispatch(Msg::RequestError(e));
                    }
                }
            };
            spawn_local(async_fetch(program))
        })
    }

    fn push_state(resource: &Resource, url: &str) {
        let history = sauron::window().history().expect("must have history");
        log::trace!("pushing to state: {}", url);
        history
            .push_state_with_url(&serde_wasm_bindgen::to_value(&resource).unwrap(), "", Some(url))
            .expect("must push state");
    }
}
