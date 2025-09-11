use std::{
    error::Error,
    fs,
};
use clap::{ArgAction, Parser};
use pmrac::platform::Builder as ACPlatformBuilder;
use pmrdb::{
    Backend,
    ConnectorOption,
};
use pmrrbac::Builder as PmrRbacBuilder;

use super::Platform;

#[derive(Clone, Debug, Default, Parser)]
pub struct Builder {
    #[clap(
        long,
        value_name = "PMR_ANONYMOUS_READER",
        env = "PMR_ANONYMOUS_READER",
        action = ArgAction::Set,
        default_value_t = true,
        default_missing_value = "true",
    )]
    pub pmr_anonymous_reader: bool,
    #[clap(
        long,
        value_name = "PMR_AUTO_CREATE_DB",
        env = "PMR_AUTO_CREATE_DB",
        action = ArgAction::Set,
        default_value_t = true,
        default_missing_value = "true",
    )]
    pub pmr_auto_create_db: bool,
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pub pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pub pmr_repo_root: String,
    #[clap(long, value_name = "PMRAC_DB_URL", env = "PMRAC_DB_URL")]
    pub pmrac_db_url: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pub pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pub pmrtqs_db_url: String,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pmr_anonymous_reader(mut self, value: bool) -> Self {
        self.pmr_anonymous_reader = value;
        self
    }

    pub fn pmr_auto_create_db(mut self, value: bool) -> Self {
        self.pmr_auto_create_db = value;
        self
    }

    pub fn pmr_data_root(mut self, value: String) -> Self {
        self.pmr_data_root = value;
        self
    }

    pub fn pmr_repo_root(mut self, value: String) -> Self {
        self.pmr_repo_root = value;
        self
    }

    pub fn pmrac_db_url(mut self, value: String) -> Self {
        self.pmrac_db_url = value;
        self
    }

    pub fn pmrapp_db_url(mut self, value: String) -> Self {
        self.pmrapp_db_url = value;
        self
    }

    pub fn pmrtqs_db_url(mut self, value: String) -> Self {
        self.pmrtqs_db_url = value;
        self
    }

    pub async fn build(self) -> Result<Platform, Box<dyn Error + Send + Sync>> {
        Ok(Platform::new(
            ACPlatformBuilder::new()
                .boxed_ac_platform(
                    Backend::ac(
                        ConnectorOption::from(&self.pmrac_db_url)
                            .auto_create_db(self.pmr_auto_create_db)
                    )
                        .await?
                )
                .pmrrbac_builder(
                    PmrRbacBuilder::new()
                        .anonymous_reader(self.pmr_anonymous_reader)
                )
                .build(),
            Backend::mc(
                ConnectorOption::from(&self.pmrapp_db_url)
                    .auto_create_db(self.pmr_auto_create_db)
            )
                .await?
                .into(),
            Backend::tm(
                ConnectorOption::from(&self.pmrtqs_db_url)
                    .auto_create_db(self.pmr_auto_create_db)
            )
                .await?
                .into(),
            fs::canonicalize(self.pmr_data_root)?,
            fs::canonicalize(self.pmr_repo_root)?,
        ))
    }
}
