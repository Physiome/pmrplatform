use pmrcore::exposure::traits::Exposure as _;
use pmrctrl::platform::Platform;
use std::{
    fs,
    path::Path,
    io::BufReader,
};

use crate::{
    cellml::cmeta::{Cmeta, Pmr2Cmeta},
    cli::Arguments,
};

pub async fn run(
    platform: &Platform,
    Arguments { input_path, output_dir, exposure_id, exposure_path, .. }: Arguments,
) -> anyhow::Result<()> {
    let resource_path = format!("/exposure/{exposure_id}/{exposure_path}");

    let reader = BufReader::new(fs::File::open(input_path)?);
    let cmeta = Cmeta::new(reader)?;

    let cmeta_id = cmeta.root_cmetaid();
    let title = cmeta.dc_title(Some(""))?;
    let citations = cmeta.citation(cmeta_id.map(|s| format!("#{s}")).as_deref())?;
    let vcards = cmeta.dc_vcard_info(Some(""))?;
    let keywords = cmeta.contextual_keywords()?;

    // first gather the data for the output file
    let mut pmr2_cmeta = Pmr2Cmeta::default();
    pmr2_cmeta.keywords = Some(keywords.clone());
    pmr2_cmeta.model_title = title.get(0).take().cloned();
    if let Some(vcards) = vcards.get(0) {
        pmr2_cmeta.model_author = vcards.fullname();
        pmr2_cmeta.model_author_org = vcards.org();
    }
    if let Some(citations) = citations.get(0) {
        pmr2_cmeta.citation_bibliographic_citation = citations.journal.clone();
        pmr2_cmeta.citation_authors = Some(citations.authors
            .iter()
            .map(|a| (
                a.family.to_string(),
                a.given.clone().unwrap_or_default(),
                a.other.clone().unwrap_or_default(),
            ))
            .collect()
        );
        pmr2_cmeta.citation_title = Some(citations.title.clone());
        pmr2_cmeta.citation_id = Some(citations.id.clone());
        pmr2_cmeta.citation_issued = citations.issued.clone();
    }
    pmr2_cmeta.citations = citations;

    // write the output to the file
    let output = fs::File::create(Path::new(&output_dir).join("cmeta.json"))?;
    serde_json::to_writer(output, &pmr2_cmeta)?;

    // with the data gathered, populate the index
    // Add the various information acquired from the metadata into the index
    platform.pc_platform.resource_link_kind_with_terms(
        &resource_path,
        "cellml_keyword",
        &mut keywords.iter()
            .map(|(_, kw)| kw.as_str()),
    )
    .await?;
    // aka title under PMR2
    platform.pc_platform.resource_link_kind_with_term(
        &resource_path,
        "description",
        // fallback from main title to citation title to generated one.
        &pmr2_cmeta.model_title.unwrap_or(
            pmr2_cmeta.citation_title.unwrap_or(
                format!("Exposure {exposure_id}")
            )
        )
    )
    .await?;
    // citation

    for citation in pmr2_cmeta.citations.iter() {
        platform.pc_platform.add_citation(&citation).await.ok();
    }
    // Citation id.
    platform.pc_platform.resource_link_kind_with_terms(
        &resource_path,
        "citation_id",
        &mut pmr2_cmeta.citations
            .iter()
            .map(|citation| citation.id.as_ref()),
    )
    .await?;
    platform.pc_platform.resource_link_kind_with_terms(
        &resource_path,
        "citation_author_family_name",
        &mut pmr2_cmeta.citations
            .iter()
            .map(|citation| {
                citation.authors
                    .iter()
                    .map(|author| author.family.as_ref())
            })
            .flatten()
    )
    .await?;

    let exposure = platform.get_exposure(exposure_id).await?;
    platform.pc_platform.resource_link_kind_with_term(
        &resource_path,
        "created_ts",
        &exposure.exposure().created_ts().to_string(),
    )
    .await?;

    let model_author_full_names = vcards.iter()
        .filter_map(|vcard| vcard.fullname())
        .collect::<Vec<_>>();
    platform.pc_platform.resource_link_kind_with_terms(
        &resource_path,
        "model_author",
        &mut model_author_full_names.iter()
            .map(String::as_str),
    )
    .await?;

    let file = exposure.ctrl_path(exposure_path).await?;
    let pathinfo = file.pathinfo();
    let repo = pathinfo.repo();
    if let Some(commit) = pathinfo.commit(&repo) {
        let seconds = commit.decode()?.author()?.time()?.seconds.to_string();
        platform.pc_platform.resource_link_kind_with_term(
            &resource_path,
            "commit_authored_ts",
            &seconds.to_string(),
        )
        .await?;
    }
    Ok(())
}
