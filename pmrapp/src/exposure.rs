use leptos::logging;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{
        Form,
        ParentRoute,
        Route,
    },
    hooks::use_params,
    nested_router::Outlet,
    params::Params,
    MatchNestedRoutes,
    ParamSegment,
    SsrMode,
    StaticSegment,
    WildcardSegment,
};
use pmrcore::{
    exposure::{
        self,
        profile::ExposureFileProfile,
    },
    profile::UserPromptGroup,
    task_template::{
        UserArg,
        UserChoice,
    },
};
use std::{
    str::FromStr,
    sync::{
        Arc,
        OnceLock,
    },
};
use wasm_bindgen::{
    JsCast,
    UnwrapThrowExt,
};

pub mod api;

use crate::{
    // ac::AccountCtx,
    component::{
        Redirect,
        RedirectTS,
        SelectList,
        SelectMap,
        Spinner,
    },
    error::AppError,
    error_template::ErrorTemplate,
    enforcement::{
        EnforcedOk,
        // PolicyState,
    },
    exposure::api::{
        list,
        get_exposure_info,
        resolve_exposure_path,
        update_wizard_field,
        wizard,
        ExposureInfo,
        WizardAddFile,
        WIZARD_FIELD_ROUTE,
    },
    view::{
        EFView,
        ExposureFileView,
    },
    app::portlet::{
        ContentActionCtx,
        ContentActionItem,
        ExposureSourceCtx,
        ExposureSourceItem,
        NavigationCtx,
        NavigationItem,
        ViewsAvailableCtx,
    },
};

mod types {
    use pmrcore::exposure::{
        ExposureFile,
        ExposureFileView,
    };

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub enum ResolvedExposurePath {
        Target(ExposureFile, Result<(ExposureFileView, Option<String>), Vec<String>>),
        Redirect(String),
    }
}

pub use types::ResolvedExposurePath;

#[component]
pub fn ExposureRoutes() -> impl MatchNestedRoutes + Clone {
    let ssr = SsrMode::Async;
    view! {
        <ParentRoute path=StaticSegment("/exposure") view=ExposureRoot ssr>
            <Route path=StaticSegment("/") view=ExposureListing/>
            <Route path=StaticSegment("") view=RedirectTS/>
            <ParentRoute path=ParamSegment("id") view=Exposure>
                <Route path=StaticSegment("/") view=ExposureMain/>
                <Route path=StaticSegment("") view=RedirectTS/>
                <Route path=(StaticSegment("+"), StaticSegment("wizard")) view=Wizard/>
                <Route path=WildcardSegment("path") view=ExposureFile/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn ExposureRoot() -> impl IntoView {
    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Outlet/>
    }
}

#[component]
pub fn ExposureListing() -> impl IntoView {
    let exposures = Resource::new_blocking(
        move || (),
        move |_| async move {
            let result = list().await;
            match result {
                Ok(ref result) => logging::log!("{}", result.inner.len()),
                Err(_) => logging::log!("error loading exposures"),
            };
            result.map(EnforcedOk::notify_into)
        },
    );
    let exposure_listing = move || Suspend::new(async move {
        exposures.await.map(|exposures| exposures
            .into_iter()
            .map(move |exposure| view! {
                <div>
                    <div><a href=format!("/exposure/{}/", exposure.id)>
                        "Exposure "{exposure.id}
                    </a></div>
                    <div>{exposure.description}</div>
                </div>
            })
            .collect_view()
        )
    });

    view! {
        <div class="main">
            <h1>"Listing of exposures"</h1>
            <div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {exposure_listing}
                </ErrorBoundary>
            </Transition>
            </div>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureParams {
    id: Option<i64>,
}

#[component]
pub fn Exposure() -> impl IntoView {
    on_cleanup(move || {
        leptos::logging::log!("on_cleanup <Exposure>");
        use_context::<WriteSignal<ExposureSourceCtx>>()
            .map(|ctx| ctx.update(ExposureSourceCtx::clear));
        use_context::<WriteSignal<NavigationCtx>>()
            .map(|ctx| ctx.update(NavigationCtx::clear));
        // FIXME when ContentAction is introduced here, use that for implicit cleanup.
        // if let Some(account_ctx) = use_context::<AccountCtx>() {
        //     leptos::logging::log!("used context AccountCtx to set_ps");
        //     account_ctx.set_ps.update(|ctx| *ctx = PolicyState::default());
        // }
    });
    let params = use_params::<ExposureParams>();
    provide_context(Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => get_exposure_info(id)
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(AppError::from),
                _ => Err(AppError::NotFound),
            }
        }
    ));
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();

    let portlets = move || {
        Suspend::new(async move {
            let exposure_info = exposure_info.await;
            let resource = exposure_info.as_ref().ok().map(|info| {
                format!("/exposure/{}/", info.exposure.id)
            });
            expect_context::<WriteSignal<ContentActionCtx>>()
                .update(|ctx| ctx.replace(resource
                    .map(|resource| {
                        on_cleanup(move || {
                            expect_context::<WriteSignal<ContentActionCtx>>().update(|ctx| {
                                ctx.reset_for("/exposure/{id}/");
                            });
                        });

                        let mut actions = vec![];
                        actions.push(ContentActionItem {
                            href: resource.clone(),
                            text: "Exposure Top".to_string(),
                            title: None,
                            req_action: None,
                        });
                        actions.push(ContentActionItem {
                            href: format!("{resource}+/wizard"),
                            text: "Wizard".to_string(),
                            title: Some("Build this exposure".to_string()),
                            req_action: Some("edit".to_string()),
                        });
                        ContentActionCtx::new("/exposure/{id}/".into(), actions)
                    })
                    .unwrap_or_default()
                ));
            expect_context::<WriteSignal<ExposureSourceCtx>>()
                .update(|ctx| ctx.replace(exposure_info.as_ref()
                    .map(|info| {
                        logging::log!("building ExposureSourceItem");
                        ExposureSourceItem {
                            commit_id: info.exposure.commit_id.clone(),
                            workspace_id: info.exposure.workspace_id.to_string(),
                            // TODO put in the workspace title.
                            workspace_title: info.workspace.description.clone().unwrap_or(
                                format!("Workspace {}", info.exposure.workspace_id)),
                        }.into()
                    })
                    .ok()
                    .into()
                ));
            expect_context::<WriteSignal<NavigationCtx>>()
                .update(|ctx| ctx.replace(exposure_info
                    .map(|info| {
                        let exposure_id = info.exposure.id;
                        logging::log!("building NavigationCtx");
                        // TODO should derive from exposure.files when it contains title/description
                        info.files
                            .into_iter()
                            .filter_map(move |(file, flag)| {
                                flag.then(|| {
                                    let href = format!("/exposure/{exposure_id}/{file}/");
                                    let text = file.clone();
                                    let title = None;
                                    NavigationItem { href, text, title }
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .ok()
                    .into()
                ));
        })
    };

    view! {
        <Title text="Exposure — Physiome Model Repository"/>
        <Suspense>
            {portlets}
        </Suspense>
        <Outlet/>
    }
}

#[component]
pub fn ExposureFileListing(id: i64, files: Vec<(String, bool)>) -> impl IntoView {
    view! {
        <ul>{files.into_iter()
            .map(|(file, flag)| view! {
                <li>
                    <a href=format!("/exposure/{id}/{file}")>
                        {file.clone()}
                    </a>
                    " - "{flag.then(|| view! {
                        <a href=format!("/exposure/{id}/{file}/")>
                            {flag}
                        </a>
                    }.into_any()).unwrap_or("false".into_any())}
                </li>
            })
            .collect_view()
        }</ul>
    }
}

#[component]
pub fn ExposureMain() -> impl IntoView {
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();
    let file_listing = move || Suspend::new(async move {
        exposure_info.await.map(|info| view! {
            <h1>"Viewing exposure "{info.exposure.id}</h1>
            <ExposureFileListing id=info.exposure.id files=info.files/>
        })
    });

    view! {
        <div class="main">
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {file_listing}
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ExposureFileParams {
    path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ViewPath(pub Option<String>);

#[component]
pub fn ExposureFile() -> impl IntoView {
    on_cleanup(|| {
        use_context::<WriteSignal<ViewsAvailableCtx>>()
            .map(|ctx| ctx.update(ViewsAvailableCtx::clear));
    });
    let params = use_params::<ExposureFileParams>();
    let exposure_info = expect_context::<Resource<Result<ExposureInfo, AppError>>>();
    let file = Resource::new_blocking(
        move || params.get().map(|p| p.path),
        move |p| async move {
            match (exposure_info.await, p) {
                (Ok(info), Ok(Some(path))) => resolve_exposure_path(info.exposure.id, path.clone())
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(|_| AppError::NotFound),
                _ => Err(AppError::InternalServerError),
            }
        }
    );

    let view_key_entry = move |(ef, view_key): (&exposure::ExposureFile, String)| view! {
        <li>
            <a href=format!("/exposure/{}/{}/{}", ef.exposure_id, ef.workspace_file_path, view_key)>
                {view_key.clone()}
            </a>
        </li>
    };

    let ep_view = move || Suspend::new(async move {
        match file.await
            .map_err(|_| AppError::NotFound)
        {
            // TODO figure out how to redirect to the workspace.
            Ok(ResolvedExposurePath::Target(ef, Ok((efv, view_path)))) => {
                expect_context::<WriteSignal<ViewsAvailableCtx>>()
                    .update(|ctx| ctx.replace((&ef).into()));
                let view_key = efv.view_key.clone();
                let view_key = EFView::from_str(&view_key
                    .expect("API failed to produce a fully formed ExposureFileView")
                )?;
                provide_context(ef);
                provide_context(efv);
                provide_context(ViewPath(view_path));
                Ok(view! {
                    <ExposureFileView view_key/>
                }.into_any())
            }
            Ok(ResolvedExposurePath::Target(ef, Err(view_keys))) => {
                expect_context::<WriteSignal<ViewsAvailableCtx>>()
                    .update(|ctx| ctx.replace((&ef).into()));
                Ok(view! {
                    <h1>
                        "Exposure "{ef.exposure_id}
                        " - ExposureFile "{ef.workspace_file_path.clone()}
                        " - Listing of all views"
                    </h1>
                    <ul>{
                        view_keys.into_iter()
                            .map(|k| view_key_entry((&ef, k)))
                            .collect_view()
                    }</ul>
                }.into_any())
            }
            Ok(ResolvedExposurePath::Redirect(path)) => {
                Ok(view! { <Redirect path show_link=true/> }.into_any())
            }
            _ => Err(AppError::NotFound),
        }
    });

    view! {
        <div class="main">
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                    {ep_view}
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[component]
pub fn WizardField(
    exposure_id: i64,
    ef_profile: impl AsRef<ExposureFileProfile> + Send + Sync + 'static,
    // Should this really be an AsRef? Typically this actually typically
    // is unique thus safe to be moved here.
    // user_arg: impl AsRef<UserArg> + Send + Sync,
    user_arg: UserArg,
)-> impl IntoView {
    let ef_profile_ref = ef_profile.as_ref();
    let field_input = ef_profile_ref.user_input.get(&user_arg.id).map(|s| s.to_string());
    let name = format!("{}-{}", ef_profile_ref.exposure_file_id, user_arg.id);
    let (v, _) = arc_signal(field_input.clone());

    // prepare for the status clear action used in the action for
    // clearing the status after the update is done.
    let status_clear = Arc::new(OnceLock::<ArcAction<(), ()>>::new());
    let status_clear_clone = status_clear.clone();

    // this is for the fadeout transition for the okay status
    let (status_okay_class, set_status_okay_class) = signal("okay".to_string());

    let action = ArcAction::new(move |(name, value): &(String, String)| {
        let name = name.to_owned();
        let value = value.to_owned();
        let status_clear = status_clear_clone.clone();
        async move {
            logging::log!("sending update to field {name:?} with {value:?}");
            let result = update_wizard_field(vec![
                ("exposure_id".to_string(), exposure_id.to_string()),
                (name, value),
            ]).await;
            // TODO only clear status if result is Ok?
            status_clear.get()
                .map(|a| a.dispatch(()));
            result
        }
    });
    let action_pending = action.pending();
    let action_result = action.value();
    let mut abort_handle = None::<ActionAbortHandle>;
    let mut current = field_input.clone();

    let new_action = action.clone();
    // actually define the status clear action, using the action.value() handle
    let _ = status_clear.set(ArcAction::new(move |()| {
        let action_result = new_action.value();
        let action_version = new_action.version();
        let action_pending = new_action.pending();
        async move {
            let version = action_version.get_untracked();
            #[cfg(not(feature = "ssr"))]
            send_wrapper::SendWrapper::new(async move {
                gloo_timers::future::TimeoutFuture::new(1000).await
            }).await;
            // only clear the result if the same version as the one got started to
            // not preemptively clear unrelated status

            if let Some(Err(_)) = action_result.get_untracked() {
            } else {
                if !action_pending.get_untracked()
                    && action_version.get_untracked() == version
                {
                    // instead of clearing the result, just update the class to
                    // trigger the fadeout if permitted.
                    // action_result.set(None);
                    set_status_okay_class.set("okay fadeout".to_string());
                }
            }

        }
    }));

    let new_action = action.clone();
    // this version simply dispatch the action after a delay
    let delayed_action = ArcAction::new(move |(name, value, delay): &(String, String, u32)| {
        let action = new_action.clone();
        let name = name.to_owned();
        let value = value.to_owned();
        let _delay = *delay;
        async move {
            #[cfg(not(feature = "ssr"))]
            send_wrapper::SendWrapper::new(async move {
                gloo_timers::future::TimeoutFuture::new(_delay).await
            }).await;
            action.dispatch((name, value))
        }
    });

    let field_element = if let Some(choices) = user_arg.choices {
        let name = name.clone();
        let options = <Vec<UserChoice>>::from(choices)
            .into_iter()
            .map(|UserChoice(choice, _)| choice)
            .collect::<Vec<_>>();
        // this is used for making sure clients with active scripting (i.e. with immediate
        // update capabilities), the expected option is selected, rather than relying on
        // the browser leaving it at a possible stale value.
        view! {
            <SelectList name options value=field_input
                on:change=move |ev| {
                    let element = ev
                        .unchecked_ref::<web_sys::Event>()
                        .target()
                        .unwrap_throw()
                        .unchecked_into::<web_sys::HtmlSelectElement>();
                    let name = element.name();
                    let value = element.value();
                    action.dispatch((name, value));
                }
                prop:value=move || v.get().unwrap_or("".to_string())
            />
        }.into_any()
    } else {
        let name = name.clone();
        view! {
            <input type="text" id=name.clone() name=name value=field_input
                prop:value=move || v.get().unwrap_or("".to_string())
                on:keyup=move |ev| {
                    let element = ev
                        .unchecked_ref::<web_sys::Event>()
                        .target()
                        .unwrap_throw()
                        .unchecked_into::<web_sys::HtmlInputElement>();
                    let value = element.value();
                    // the keyup can be triggered by navigating within the
                    // field, so validate the content has in fact changed.
                    if Some(&value) != current.as_ref() {
                        let name = element.name();
                        // abort the existing abort handle, if any
                        abort_handle
                            .take()
                            .map(ActionAbortHandle::abort);
                        // record the update here while also dispatch the
                        // action with a small delay for the newly set
                        // abort handle to repeat the cycle, effectively
                        // function as a debouncer.
                        current = Some(value.clone());
                        abort_handle = Some(delayed_action.dispatch((name, value, 500)));
                    }
                }
            />
        }.into_any()
    };
    view! {
        <label for=name>
            {user_arg.prompt}
            <div class="status">{move ||
                if action_pending.get() {
                    Some(view! {
                        <Spinner/>
                    }.into_any())
                }
                else if let Some(result) = action_result.get() {
                    Some(match result {
                        Ok(_) => {
                            set_status_okay_class.set("okay".to_string());
                            view! {
                                <div class=move || status_okay_class.get()
                                    aria-label="field updated">"✔"</div>
                            }.into_any()
                        },
                        Err(e) => view! {
                            <div class="error">{format!("Error: {e}")}</div>
                        }.into_any(),
                    })
                }
                else {
                    None
                }
            }</div>
        </label>
        {field_element}
    }
}

#[component]
pub fn Wizard() -> impl IntoView {
    let wizard_add_file = ServerAction::<WizardAddFile>::new();

    let params = use_params::<ExposureParams>();
    let wizard_res = Resource::new_blocking(
        move || params.get().map(|p| p.id),
        |p| async move {
            match p {
                Err(_) => Err(AppError::InternalServerError),
                Ok(Some(id)) => wizard(id)
                    .await
                    .map(EnforcedOk::notify_into)
                    .map_err(AppError::from),
                _ => Err(AppError::NotFound),
            }
        }
    );

    let wizard_view = move || Suspend::new(async move {
        wizard_res.await.map(|info| {
            let unassigned_files = info.files.iter()
                .filter_map(|(name, status)| status.is_none().then_some(name.clone()))
                .collect::<Vec<_>>();
            let profile_map = info.profiles.iter()
                .map(|v| (v.id.to_string(), v.title.clone()))
                .collect::<Vec<_>>();

            let add_file_form = view! {
                <ActionForm attr:class="standard" action=wizard_add_file>
                    {move || {
                        let value = wizard_add_file.value();
                        match value.get() {
                            Some(Ok(_)) => {
                                value.set(None);
                                wizard_res.refetch();
                                expect_context::<Resource<Result<ExposureInfo, AppError>>>()
                                    .refetch();
                            }
                            _ => (),
                        }
                    }}
                    <fieldset>
                        <legend>"New Exposure File"</legend>
                        <input type="hidden" name="exposure_id" value=info.exposure.id/>
                        <div>
                            <label for="path">"File"</label>
                            <SelectList
                                name="path".to_string()
                                options=unassigned_files />
                        </div>
                        <div>
                            <label for="profile_id">"File Type"</label>
                            <SelectMap
                                name="profile_id".to_string()
                                options=profile_map />
                        </div>
                        <div>
                            <button type="submit">"Create Exposure File"</button>
                        </div>
                    </fieldset>
                </ActionForm>
            };

            let exposure_id = info.exposure.id;

            let files_view = info.files.into_iter()
                .filter_map(|(name, value)| {
                    value.map(|(ef_profile, user_prompt_groups)| {
                        let ef_profile = Arc::new(ef_profile);
                        let user_prompt_groups: Vec<UserPromptGroup> = user_prompt_groups.into();
                        let group_views = user_prompt_groups.into_iter()
                            .map(|group| {
                                let user_args: Vec<UserArg> = group.user_args.into();
                                let fields = user_args.into_iter()
                                    .map(|user_arg| {
                                        view! {
                                            <WizardField
                                                exposure_id
                                                user_arg=user_arg
                                                ef_profile=ef_profile.clone()
                                                />
                                        }
                                    })
                                    .collect_view();
                                view! {
                                    <fieldset>
                                        <legend>{group.description}</legend>
                                        {fields}
                                    </fieldset>
                                }
                            })
                            .collect_view();

                        view! {
                            <fieldset>
                                <legend>"Configuration for: "{name}</legend>
                                {group_views}
                            </fieldset>
                        }
                    })
                })
                .collect_view();

            view! {
                {add_file_form}
                <Form
                    attr:class="standard"
                    action=WIZARD_FIELD_ROUTE
                    method="post"
                >
                    <input type="hidden" name="exposure_id" value=info.exposure.id/>
                    <fieldset>
                        <legend>"Exposure Files"</legend>
                        {files_view}
                        <div>
                            <button type="submit">"Update"</button>
                        </div>
                    </fieldset>
                </Form>
            }
        })
    });

    view! {
        <h1>"Exposure Wizard"</h1>
        <Transition>
            {wizard_view}
        </Transition>
    }
}
