//! Applies the J-102 deterministic admission scan to the E-43 skill surface.
//! The scanner is `elai_core::admission::scan_instruction_surface` and is NOT
//! re-implemented here.

use super::SkillManifest;
use elai_core::admission::{scan_instruction_surface, AdmissionVerdict};

/// Return only skill manifests whose source text can be admitted safely.
pub fn admitted_skills(manifests: Vec<SkillManifest>) -> Vec<SkillManifest> {
    manifests
        .into_iter()
        .filter_map(|manifest| {
            let text = match std::fs::read_to_string(&manifest.file_path) {
                Ok(text) => text,
                Err(error) => {
                    tracing::warn!(
                        skill = %manifest.name,
                        path = %manifest.file_path.display(),
                        error = %error,
                        "skill admission refused: source could not be read"
                    );
                    return None;
                }
            };
            match scan_instruction_surface(&text) {
                AdmissionVerdict::Admitted => Some(manifest),
                AdmissionVerdict::Quarantined(findings) => {
                    tracing::warn!(
                        skill = %manifest.name,
                        path = %manifest.file_path.display(),
                        ?findings,
                        "skill admission refused: instruction surface quarantined"
                    );
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::admitted_skills;
    use crate::skill_registry::{SkillManifest, SkillOrigin};
    use std::path::PathBuf;

    fn manifest(name: &str, file_path: PathBuf) -> SkillManifest {
        SkillManifest {
            name: name.to_owned(),
            version: "0.0.0".to_owned(),
            triggers: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path,
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        }
    }

    #[test]
    fn keeps_benign_skill() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("SKILL.md");
        std::fs::write(&path, "---\nname: benign\n---\nplain body\n").expect("write");
        let kept = admitted_skills(vec![manifest("benign", path)]);
        assert_eq!(kept.len(), 1);
    }

    #[test]
    fn drops_scanner_trip_text() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("SKILL.md");
        let hidden = char::from_u32(0x200B).expect("zero-width character");
        std::fs::write(&path, format!("plain a{hidden}b body")).expect("write");
        assert!(admitted_skills(vec![manifest("hidden", path)]).is_empty());
    }

    #[test]
    fn drops_missing_skill() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("missing").join("SKILL.md");
        assert!(admitted_skills(vec![manifest("missing", path)]).is_empty());
    }

    #[test]
    fn preserves_survivor_order() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let first = tmp.path().join("first.md");
        let second = tmp.path().join("second.md");
        let third = tmp.path().join("third.md");
        std::fs::write(&first, "first").expect("write");
        std::fs::write(&second, "second").expect("write");
        let result = admitted_skills(vec![
            manifest("first", first),
            manifest("missing", tmp.path().join("none")),
            manifest("second", second),
            manifest("third", third),
        ]);
        assert_eq!(result.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["first", "second"]);
    }
}
