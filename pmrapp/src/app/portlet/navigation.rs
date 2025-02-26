use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

// Even if the Resource wrapping this is optional, the inner can still be
// None to help with error while processing the resource, and have that be
// distinct from a thing that offers no additional pages if the goal is to
// also keep the portlet visible.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationCtx(pub Option<Vec<NavigationItem>>);

impl NavigationCtx {
    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn set(&mut self, value: Vec<NavigationItem>) {
        self.0 = Some(value);
    }

    pub fn replace(&mut self, value: Self) {
        self.0 = value.0;
    }
}

#[component]
pub fn Navigation() -> impl IntoView {
    use_context::<ReadSignal<Resource<NavigationCtx>>>().map(move |ctx| {
        let resource = ctx.get();
        view! {
            <Transition>{
                move || Suspend::new(async move {
                    resource.await.0.map(|navigation| {
                        let view = navigation.into_iter()
                            .map(|NavigationItem { href, text, .. }| view! {
                                <li><A href>{text}</A></li>
                            })
                            .collect_view();
                        view! {
                            <section>
                                <h4>"Navigation"</h4>
                                <nav>
                                    <ul>
                                        {view}
                                    </ul>
                                </nav>
                            </section>
                        }
                    })
                })
            }</Transition>
        }
    })
}

impl From<Vec<NavigationItem>> for NavigationCtx {
    fn from(item: Vec<NavigationItem>) -> Self {
        Self(Some(item))
    }
}

impl From<Option<Vec<NavigationItem>>> for NavigationCtx {
    fn from(item: Option<Vec<NavigationItem>>) -> Self {
        Self(item)
    }
}
