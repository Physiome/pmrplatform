use clap::Subcommand;
use html2text::render::TrivialDecorator;
use pmrctrl::platform::Platform;
use std::io::{Read, Write};

use crate::{
    cellml::{
        cmeta::Cmeta,
        legacy::sub_makefile_terms,
    },
    cli::Arguments,
    xml::Xml,
};

mod htmldoc;

#[derive(Debug, Subcommand)]
pub enum Docgen {
    #[command(arg_required_else_help = true)]
    Tmpdoc(Arguments),
    #[command(arg_required_else_help = true)]
    Rst(Arguments),
    #[command(arg_required_else_help = true)]
    Htmldoc(Arguments),
}


impl Docgen {
    pub async fn run(self, platform: &Platform) -> anyhow::Result<()> {
        let (arguments, title, doc) = match self {
            Docgen::Tmpdoc(arguments) => {
                let reader = arguments.input_reader()?;
                let xml = Xml::new(reader)?;
                let doc = sub_makefile_terms(&xml.xslt()?);
                let reader = arguments.input_reader()?;

                // Leverage Cmeta for the title.
                let cmeta = Cmeta::new(reader)?;
                let cmeta_id = cmeta.root_cmetaid();
                let model_title = cmeta.dc_title(Some(""))?
                    .get(0)
                    .take()
                    .cloned();
                let citation_title = cmeta
                    .citation(cmeta_id.map(|s| format!("#{s}")).as_deref())?
                    .get(0)
                    .map(|citation| citation.title.clone());

                // based on the cli cmeta extraction.
                let title = model_title.unwrap_or(
                    citation_title.unwrap_or(arguments.exposure_path.clone())
                );

                (arguments, Some(title), Some(doc))
            }
            Docgen::Rst(arguments) => {
                let mut reader = arguments.input_reader()?;
                let mut buffer = String::new();
                reader.read_to_string(&mut buffer)?;
                // let doc = parserst::parse(&buffer)?
                //     .into_iter()
                //     .map(|b| b.to_string())
                //     .collect::<Vec<_>>()
                //     .join("\n");
                let rstdoc = rst_parser::parse(&buffer)?;
                let mut stream = Vec::<u8>::new();
                rst_renderer::render_html(&rstdoc, &mut stream, false)?;
                let doc = String::from_utf8(stream)?;
                // TODO placeholder.
                let title = arguments.exposure_path.clone();
                (arguments, Some(title), Some(doc))
            }
            Docgen::Htmldoc(arguments) => {
                let reader = arguments.input_reader()?;
                let title = htmldoc::parse_title(reader)?;
                let mut doc = String::new();
                arguments.input_reader()?
                    .read_to_string(&mut doc)?;
                (arguments, Some(title), Some(doc))
            }
        };

        // write the output to the file
        let text = if let Some(doc) = doc {
            if !arguments.dry_run {
                let mut output_writer = arguments.output_writer("index.html")?;
                output_writer.write(doc.as_bytes())?;
            }
            let htmlconf = html2text::config::with_decorator(TrivialDecorator::new())
                .raw_mode(true);
            let text = htmlconf.string_from_read(doc.as_bytes(), usize::MAX - 1)?;
            Some(text)
        } else {
            None
        };

        let resource_path = arguments.resource_path();
        if !arguments.dry_run {
            platform.index_backend.add_idx_text(title.as_deref(), text.as_deref(), &resource_path).await?;
        } else {
            println!("*** DRY RUN ***");
            println!("<{resource_path}>");
            if let Some(title) = title {
                println!("Title: {title}");
            } else {
                println!("Title: <untitled>");
            }
            println!("----");
            if let Some(text) = text {
                println!("{text}");
            } else {
                println!("<blank>");
            }
        }

        // platform.index_backend.resource_link_kind_with_terms(
        //     &resource_path,
        //     "text",
        //     &mut text.as_str()
        //         .split_whitespace(),
        // )
        // .await?;

        Ok(())
    }
}
