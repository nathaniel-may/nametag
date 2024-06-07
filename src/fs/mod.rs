use crate::{
    error::{Error, Result},
    schema::Schema,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

// pub fn read_schema_file(path: &Path) -> Result<Schema> {
//     let contents = fs::read_to_string(path).map_err(Error::FailedToReadContents)?;
//     let parsed = schema::parse::parse(&contents)?;
//     let schema = schema::typecheck::typecheck(parsed)?;
//     Ok(schema)
// }

pub fn read_schema_file(path: &Path) -> Result<Schema> {
    let contents = fs::read_to_string(path).map_err(Error::FailedToReadContents)?;
    let schema = serde_dhall::from_str(&contents).parse().unwrap(); // TODO map error type
    Ok(schema)
}

#[cfg(test)]
#[test]
fn init_config_file_parses() {
    use crate::schema::Category;
    use crate::schema::Requirement::*;

    let expected = Schema {
        delim: "-".to_string(),
        categories: vec![
            Category {
                name: "Medium".to_string(),
                rtype: Exactly,
                rvalue: 1,
                values: vec![
                    "art".to_string(),
                    "photo".to_string(),
                    "ai".to_string(),
                    "other".to_string(),
                ],
            },
            Category {
                name: "Subject".to_string(),
                rtype: AtLeast,
                rvalue: 0,
                values: vec![
                    "plants".to_string(),
                    "animals".to_string(),
                    "people".to_string(),
                ],
            },
        ],
    };

    match read_schema_file(Path::new("./src/init.dhall")) {
        Err(e) => panic!("{e}"),
        Ok(schema) => assert_eq!(expected, schema),
    }
}

/// collects filenames of all non-directory entries in the given directory.
pub fn collect_filenames(dir: &dyn AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = vec![];
    let dir = fs::read_dir(dir).map_err(Error::CantOpenWorkingDir)?;
    for path in dir {
        let entry = path.map_err(Error::WorkingDirScan)?;
        // skip sub directories
        if !entry.path().is_dir() {
            files.push(entry.path());
        }
    }

    Ok(files)
}

#[cfg(test)]
/// used to test file system limitations for cross-platform compatibility
mod limitations {
    #[test]
    fn filename_length() {
        let char_255 = [
            "Lorem_ipsum_dolor_sit_amet_consectetur_adipiscing_elit_sed_do_eiusmod_tempor_incididunt_ut_labore_et_dolore_magna_aliqua_Ut_enim_ad_minim_veniam_quis_nostrud_exercitation_ullamco_laboris_nisi_ut_aliquip_ex_ea_commodo_consequat_Duis_aute_irure_dolor_in.jpg",
            "子曰。學而時習之、不亦說乎。 有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而道生。孝弟也者、其爲仁之本與。子曰。巧言令色、鮮矣仁。曾子曰。吾日三省吾身、爲人謀而不忠乎。與朋友交而不信乎。傳不習乎。子曰。道千乘之國、敬事而信、節用而愛人。使民以時。子曰。弟子、入則孝、出則弟、謹而信、凡愛衆、而親仁。行有餘力、則以學文。子夏曰。賢賢易色、事父母、能竭其力、事君、能致其身、與朋友交、言而有信。雖曰未學、吾必謂之學.jpg",
        ];
        for s in char_255 {
            assert_eq!(255, s.chars().count());
        }

        let utf8_byte_255 = [
            "Lorem_ipsum_dolor_sit_amet_consectetur_adipiscing_elit_sed_do_eiusmod_tempor_incididunt_ut_labore_et_dolore_magna_aliqua_Ut_enim_ad_minim_veniam_quis_nostrud_exercitation_ullamco_laboris_nisi_ut_aliquip_ex_ea_commodo_consequat_Duis_aute_irure_dolor_in.jpg",
            "子曰。學而時習之、不亦說乎。有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而--.jpg",
        ];
        for s in utf8_byte_255 {
            assert_eq!(255, s.bytes().len());
        }

        let utf16_byte_254 = [
            "Lorem_ipsum_dolor_sit_amet_consectetur_adipiscing_elit_sed_do_eiusmod_tempor_incididunt_ut_labore_et_dolore_magna_aliqua_Ut.jpg",
            "子曰。學而時習之、不亦說乎。 有朋自遠方來、不亦樂乎。人不知而不慍、不亦君子乎。有子曰。其爲人也孝弟、而好犯上者、鮮矣。不好犯上、而好作亂者、未之有也。君子務本、本立而道生。孝弟也者、其爲仁之本與。子曰。巧言令色、鮮矣仁。曾子曰。吾日三省吾身、爲.jpg"
        ];
        for s in utf16_byte_254 {
            assert_eq!(254, s.encode_utf16().count() * 2);
        }
    }
}
