use const_format::formatcp;
use std::{
    collections::BTreeMap,
    sync::LazyLock,
};

static MAKEFILE_TERMS: LazyLock<BTreeMap<&str, &str>> = LazyLock::new(|| {
    const SITEROOT: &str = "https://www.cellml.org/models";
    BTreeMap::from([
        ("HTML_EXMPL_ALBRECHT_MODEL1", formatcp!("{SITEROOT}/albrecht_colegrove_hongpaisan_pivovarova_andrews_friel_2001_version01")),
        ("HTML_EXMPL_B_SAN_MODEL", formatcp!("{SITEROOT}/boyett_zhang_garny_holden_2001_version01")),
        ("HTML_EXMPL_BAKKER_MODEL", formatcp!("{SITEROOT}/bakker_michels_opperdoes_westerhoff_1997_version01")),
        ("HTML_EXMPL_BAKKER_MODEL2", formatcp!("{SITEROOT}/bakker_mensonides_teusink_vanhoek_michels_westerhoff_2000_version01")),
        ("HTML_EXMPL_BERNUS_MODEL", formatcp!("{SITEROOT}/bernus_wilders_zemlin_verschelde_panfilov_2002_version01")),
        ("HTML_EXMPL_BERTRAM_MODEL", formatcp!("{SITEROOT}/bertram_previte_sherman_kinard_satin_2000_version02")),
        ("HTML_EXMPL_BERTRAM_MODEL04", formatcp!("{SITEROOT}/bertram_satin_zhang_smolen_sherman_2004_version01")),
        ("HTML_EXMPL_BI_EGF_INTRO", "https://www.cellml.org/examples/examples/signal_transduction_models/bi_egf_pathway_1999/index.html"),
        ("HTML_EXMPL_BONHOEFFER_MODEL", formatcp!("{SITEROOT}/bonhoeffer_rembiszewski_ortiz_nixon_2000_version03")),
        ("HTML_EXMPL_BR_MODEL", formatcp!("{SITEROOT}/beeler_reuter_1977_version05")),
        ("HTML_EXMPL_CHAY_MODEL", formatcp!("{SITEROOT}/chay_lee_fan_1995_version02")),
        ("HTML_EXMPL_CHAY_MODEL97", formatcp!("{SITEROOT}/chay_1997_version04")),
        ("HTML_EXMPL_CHEN_MODEL", formatcp!("{SITEROOT}/chen_csikasz-nagy_gyorffy_val_novak_tyson_2000_version02")),
        ("HTML_EXMPL_CILIBERTO_MODEL", formatcp!("{SITEROOT}/ciliberto_petrus_tyson_sible_2003_version01")),
        ("HTML_EXMPL_CILIBERTO_MODEL2", formatcp!("{SITEROOT}/ciliberto_novak_tyson_2003_version01")),
        ("HTML_EXMPL_COLEGROVE_MODEL", formatcp!("{SITEROOT}/colegrove_albrecht_friel_2000_version01")),
        ("HTML_EXMPL_D_SAN_MODEL", formatcp!("{SITEROOT}/demir_clark_giles_murphey_1994_version01")),
        ("HTML_EXMPL_D99_SAN_MODEL", formatcp!("{SITEROOT}/demir_clark_giles_1999_version03")),
        ("HTML_EXMPL_DFN_MODEL", formatcp!("{SITEROOT}/difrancesco_noble_1985_version06")),
        ("HTML_EXMPL_DOKOS_MODEL_II", formatcp!("{SITEROOT}/dokos_celler_lovell_1996_version06")),
        ("HTML_EXMPL_DOKOS_MODEL", formatcp!("{SITEROOT}/dokos_celler_lovell_1996_version06")),
        ("HTML_EXMPL_DR_MODEL", formatcp!("{SITEROOT}/drouhard_roberge_1987_version02")),
        ("HTML_EXMPL_DUMAINE_MODEL", formatcp!("{SITEROOT}/dumaine_towbin_brugada_vatta_nesterenko_nesterenko_brugada_brugada_antzelevitch_1999_version01")),
        ("HTML_EXMPL_ERYTHROCYTE_METABOLISM", formatcp!("{SITEROOT}/mulquiney_kuchel_1999_version11")),
        ("HTML_EXMPL_FN_SIMPLE", formatcp!("{SITEROOT}/fitzhugh_1961_version04")),
        ("HTML_EXMPL_FRIEL_MODEL", formatcp!("{SITEROOT}/friel_1995_version01")),
        ("HTML_EXMPL_GALL_MODEL", formatcp!("{SITEROOT}/gall_susa_1999_version03")),
        ("HTML_EXMPL_GASTRIC_H_K_ATPASE_MODEL", formatcp!("{SITEROOT}/joseph_zavros_merchant_kirschner_2003_version01")),
        ("HTML_EXMPL_GLYCOLYSIS_1997", formatcp!("{SITEROOT}/rizzi_baltes_reuss_1997_version01")),
        ("HTML_EXMPL_GLYCOLYSIS", formatcp!("{SITEROOT}/rizzi_baltes_reuss_1997_version01")),
        ("HTML_EXMPL_GOLDBETER_MODEL", formatcp!("{SITEROOT}/goldbeter_1991_version04")),
        ("HTML_EXMPL_GRAPHICAL_NOTATION", "https://www.cellml.org/tutorial/notation"),
        ("HTML_EXMPL_GROSSMAN_MODEL", formatcp!("{SITEROOT}/grossman_feinberg_kuznetsov_dimitrov_paul_1998_version01")),
        ("HTML_EXMPL_HERZ_MODEL", formatcp!("{SITEROOT}/herz_bonhoeffer_anderson_may_nowak_1996_version01")),
        ("HTML_EXMPL_HHSA_INTRO", formatcp!("{SITEROOT}/hodgkin_huxley_1952_version07")),
        ("HTML_EXMPL_HMN_SIMPLE", formatcp!("{SITEROOT}/hunter_mcnaughton_noble_1975_version02")),
        ("HTML_EXMPL_IP3_CA2+_MODEL", formatcp!("{SITEROOT}/deyoung_keizer_1992_version03")),
        ("HTML_EXMPL_JRW_MODEL", formatcp!("{SITEROOT}/jafri_rice_winslow_1998_version03")),
        ("HTML_EXMPL_KEIZER_MODEL", formatcp!("{SITEROOT}/keizer_levine_1996_version03")),
        ("HTML_EXMPL_LAMBETH_MODEL", formatcp!("{SITEROOT}/lambeth_kushmerick_2002_version01")),
        ("HTML_EXMPL_LI_MODEL", formatcp!("{SITEROOT}/li_rinzel_1994_version02")),
        ("HTML_EXMPL_LR_I_MODEL", formatcp!("{SITEROOT}/luo_rudy_1991_version04")),
        ("HTML_EXMPL_LR_II_MODEL", formatcp!("{SITEROOT}/luo_rudy_1994_version02")),
        ("HTML_EXMPL_MAGNUS_MODEL", formatcp!("{SITEROOT}/magnus_keizer_1998_version01")),
        ("HTML_EXMPL_MAPK_CASCADE", formatcp!("{SITEROOT}/huang_ferrell_1996_version03")),
        ("HTML_EXMPL_MARTINOV_MODEL", formatcp!("{SITEROOT}/martinov_vitvitsky_mosharov_banerjee_ataullakhanov_2000_version01")),
        ("HTML_EXMPL_MITOCHONDRIAL_CA_HANDLING", formatcp!("{SITEROOT}/magnus_keizer_1997_version01")),
        ("HTML_EXMPL_MITTLER_MODEL", formatcp!("{SITEROOT}/mittler_sulzer_neumann_perelson_1998_version01")),
        ("HTML_EXMPL_MNT_MODEL", formatcp!("{SITEROOT}/mcallister_noble_tsien_1975_version05")),
        ("HTML_EXMPL_MOONEY_RIVLIN_LAW", formatcp!("{SITEROOT}/rivlin_saunders_1951_version01")),
        ("HTML_EXMPL_N_MODEL", formatcp!("{SITEROOT}/noble_1962_version05")),
        ("HTML_EXMPL_N98_MODEL", formatcp!("{SITEROOT}/noble_varghese_kohl_noble_1998_version08")),
        ("HTML_EXMPL_NOVAK_MODEL", formatcp!("{SITEROOT}/novak_tyson_1997_version01")),
        ("HTML_EXMPL_NOVAK_MODEL98", formatcp!("{SITEROOT}/novak_csikasz-nagy_gyorffy_chen_tyson_1998_version01")),
        ("HTML_EXMPL_NOWAK_MODEL", formatcp!("{SITEROOT}/nowak_bangham_1996_version03")),
        ("HTML_EXMPL_OXIDATIVE_PHOSPHORYLATION", formatcp!("{SITEROOT}/beard_2005_version01")),
        ("HTML_EXMPL_PERELSON_MODEL", formatcp!("{SITEROOT}/perelson_neumann_markowitz_leonard_ho_1996_version01")),
        ("HTML_EXMPL_PRIEBE_MODEL", formatcp!("{SITEROOT}/priebe_beuckelmann_1998_version01")),
        ("HTML_EXMPL_RICE_MODEL", formatcp!("{SITEROOT}/rice_winslow_jafri_1999_version03")),
        ("HTML_EXMPL_RICE_MODEL2", formatcp!("{SITEROOT}/rice_jafri_winslow_2000_version02")),
        ("HTML_EXMPL_RJW_MODEL", formatcp!("{SITEROOT}/rice_jafri_winslow_2000_version02")),
        ("HTML_EXMPL_SNEYD_MODEL", formatcp!("{SITEROOT}/sneyd_dufour_2002_version05")),
        ("HTML_EXMPL_SNYDER_MODEL", formatcp!("{SITEROOT}/snyder_palmer_moore_2000_version01")),
        ("HTML_EXMPL_SOBIE_MODEL", formatcp!("{SITEROOT}/sobie_dilly_dossantoscruz_lederer_jafri_2002_version01")),
        ("HTML_EXMPL_STERN_MODEL", formatcp!("{SITEROOT}/stern_song_sham_yang_boheler_rios_1999_version02")),
        ("HTML_EXMPL_TEN_TUSSCHER_MODEL04", formatcp!("{SITEROOT}/tentusscher_noble_noble_panfilov_2004_version05")),
        ("HTML_EXMPL_TEUSINK_MODEL", formatcp!("{SITEROOT}/teusink_passarge_reijenga_esgalhado_vanderweijden_schepper_walsh_bakker_vandam_westerhoff_snoep_2000_version03")),
        ("HTML_EXMPL_UPDATED_LR_II_MODEL", formatcp!("{SITEROOT}/luo_rudy_1994_version02")),
        ("HTML_EXMPL_W_MODEL", formatcp!("{SITEROOT}/winslow_rice_jafri_marban_ororke_1999_version03")),
        ("HTML_EXMPL_WODARZ_MODEL", formatcp!("{SITEROOT}/wodarz_nowak_1999_version01")),
        ("HTML_EXMPL_WOLF_HEINRICH_MODEL", formatcp!("{SITEROOT}/wolf_heinrich_2000_version02")),
        ("HTML_EXMPL_WOLF_MODEL", formatcp!("{SITEROOT}/wolf_passarge_somsen_snoep_heinrich_westerhoff_2000_version02")),
        ("HTML_EXMPL_Z_SAN_MODEL", formatcp!("{SITEROOT}/zhang_holden_kodama_honjo_lei_varghese_boyett_2000_version03")),
        ("HTML_METADATA_20020116_OVERVIEW", "https://www.cellml.org/specifications/archive/metadata/20020116/cellml_metadata_specification.pdf/view"),
        ("HTML_REPOSITORY_INTRODUCTION", formatcp!("{SITEROOT}")),
        ("HTML_SPEC_20010810_GROUPING", "https://www.cellml.org/specifications/cellml_1.1/#sec_grouping"),
        ("HTML_SPEC_20010810_UNITS", "https://www.cellml.org/specifications/cellml_1.1/#sec_units"),
        ("HTML_XML_EXMPL_GUIDE", "https://www.cellml.org/tutorial/xml_guide"),
    ])
});

pub fn sub_makefile_terms(s: &str) -> String {
    envsubst(s, &MAKEFILE_TERMS)
}

// This is a naive envsubst implementation with minimal error handling as it assumes the input is well-formed,
// as this targets just the specific use case in the legacy Makefile based CellML Model Repository.
fn envsubst(s: &str, env: &BTreeMap<&str, &str>) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    // rather than naively using the `.replace()`, use its underlying functions and apply the
    // replacement in one go.
    for (start, s_part) in s.match_indices("${") {
        // arises if a `${` is nested inside `${}`; simply ignore it as it's invalid.
        if last_end > start {
            continue
        }
        result.push_str(&s[last_end..start]);
        if let Some((idx, e_part)) = s[start..s.len()].match_indices("}").next() {
            let b_start = start + s_part.len();
            let b_end = start + idx;
            if let Some(replacement) = env.get(&s[b_start..b_end]) {
                result.push_str(replacement);
                last_end = b_end + e_part.len();
            } else {
                let pushed = &s[start..=b_end];
                result.push_str(pushed);
                last_end = start + pushed.len();
            }
        } else {
            // no ending `"}"` so we may terminate early and skip any further handling of `${`.
            break;
        }
    }
    result.push_str(&s[last_end..s.len()]);
    result

}

#[cfg(test)]
mod test {
    use std::{
        collections::BTreeMap,
        sync::LazyLock,
    };
    use super::*;

    static TERMS: LazyLock<BTreeMap<&str, &str>> = LazyLock::new(|| {
        BTreeMap::from([
            ("PLACE", "world"),
            ("NAME", "John"),
        ])
    });

    #[test]
    fn envsubst_basic() {
        assert_eq!(envsubst("${NAME}", &TERMS), "John");
        assert_eq!(envsubst("${NAME}${NAME}", &TERMS), "JohnJohn");
        assert_eq!(envsubst("[${PLACE}]", &TERMS), "[world]");
        assert_eq!(envsubst("hello ${PLACE}!", &TERMS), "hello world!");
        assert_eq!(envsubst("welcome to ${PLACE}, ${NAME}", &TERMS), "welcome to world, John");
        assert_eq!(envsubst("${ABSENT}", &TERMS), "${ABSENT}");
        assert_eq!(envsubst("${NAME}'s ${PLACE} is ${ABSENT}", &TERMS), "John's world is ${ABSENT}");
    }

    #[test]
    fn envsubst_non_terminated() {
        assert_eq!(envsubst("${NAME", &TERMS), "${NAME");
        assert_eq!(envsubst("$NAME}", &TERMS), "$NAME}");
        assert_eq!(envsubst("{NAME}", &TERMS), "{NAME}");
        assert_eq!(envsubst("${NAME is malformed.", &TERMS), "${NAME is malformed.");
        assert_eq!(envsubst("${NAME${NAME}}", &TERMS), "${NAME${NAME}}");
        assert_eq!(envsubst("${NAME${NAME}", &TERMS), "${NAME${NAME}");
        assert_eq!(envsubst("${NAME${NAME", &TERMS), "${NAME${NAME");
    }
}
