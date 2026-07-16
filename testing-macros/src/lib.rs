extern crate proc_macro;
use std::str::FromStr;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn sqlite_pcb_cache_test_case(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::from_str(
        r#"
        #[test_case(
            SqliteBackend::pc("sqlite::memory:".into())
                .await
                .unwrap()
            ; "base sqlite backend"
        )]
        #[test_case(
            CachedIndexBackend::new(
                SqliteBackend::pc("sqlite::memory:".into())
                    .await
                    .unwrap()
                    .into()
            )
            ; "disk cached sqlite backend"
        )]
        #[test_case(
            ResourceKindedTermsCache::new(
                SqliteBackend::pc("sqlite::memory:".into())
                    .await
                    .unwrap()
                    .into()
            )
            ; "memory cached sqlite backend"
        )]
        #[test_case(
            ResourceKindedTermsCache::new(
                CachedIndexBackend::new(
                    SqliteBackend::pc("sqlite::memory:".into())
                        .await
                        .unwrap()
                    .into()
                )
                .into()
            )
            ; "memory plus disk cached sqlite backend"
        )]
        "#
    )
    .unwrap();
    tokens.extend(item);
    tokens
}
