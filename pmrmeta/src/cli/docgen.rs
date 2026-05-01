use clap::Subcommand;
use std::{
    fs,
    io::BufReader,
};

use crate::{
    cellml::legacy::sub_makefile_terms,
    cli::Arguments,
    xml::Xml,
};

#[derive(Debug, Subcommand)]
pub enum Docgen {
    #[command(arg_required_else_help = true)]
    Tmpdoc(Arguments),
    #[command(arg_required_else_help = true)]
    Rst(Arguments),
}


impl Docgen {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Docgen::Tmpdoc(Arguments { input_path, output_dir, exposure_id, exposure_path }) => {
                let reader = BufReader::new(fs::File::open(input_path)?);
                let xml = Xml::new(reader)?;
                let output = sub_makefile_terms(&xml.xslt()?);
                println!("{output}");
            }
            Docgen::Rst(Arguments { input_path, output_dir, exposure_id, exposure_path }) => {
                todo!()
            }
        }
        Ok(())
    }
}
