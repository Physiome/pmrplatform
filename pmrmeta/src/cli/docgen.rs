use clap::Subcommand;
use pmrctrl::platform::Platform;
use std::io::{Read, Write};

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
    pub async fn run(self, platform: &Platform) -> anyhow::Result<()> {
        let (arguments, doc) = match self {
            Docgen::Tmpdoc(arguments) => {
                let reader = arguments.input_reader()?;
                let xml = Xml::new(reader)?;
                let doc = sub_makefile_terms(&xml.xslt()?);
                (arguments, doc)
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
                (arguments, doc)
            }
        };

        // write the output to the file
        let mut output_writer = arguments.output_writer("index.html")?;
        output_writer.write(doc.as_bytes())?;

        let htmlconf = html2text::config::with_decorator(html2text::render::TrivialDecorator::new())
            .raw_mode(true);

        let text = htmlconf.string_from_read(doc.as_bytes(), usize::MAX - 1)?;
        println!("{text}");

        let resource_path = arguments.resource_path();
        platform.pc_platform.resource_link_kind_with_terms(
            &resource_path,
            "text",
            &mut text.as_str()
                .split_whitespace(),
        )
        .await?;

        Ok(())
    }
}
