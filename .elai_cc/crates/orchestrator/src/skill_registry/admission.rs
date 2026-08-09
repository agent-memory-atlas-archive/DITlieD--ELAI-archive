//! Applies the J-102 deterministic admission scan to the E-43 skill surface.
//! The scanner is `elai_core::admission::scan_instruction_surface` and is NOT
//! re-implemented here.

use super::SkillManifest;
use elai_core::admission::{
    scan_instruction_surface, AdmissionFinding, AdmissionRule, AdmissionSeverity, AdmissionVerdict,
};
use std::path::{Path, PathBuf};

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


/// Return quarantined files found anywhere beneath a skill directory.
#[derive(Debug)]
pub struct SkillDirFinding {
    pub file: PathBuf,
    pub findings: Vec<AdmissionFinding>,
}

/// Scan every readable text file in a skill directory recursively.
pub fn scan_skill_dir(dir: &Path) -> Vec<SkillDirFinding> {
    fn walk(path: &Path, findings: &mut Vec<SkillDirFinding>) {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) => {
                findings.push(SkillDirFinding {
                    file: path.to_path_buf(),
                    findings: vec![AdmissionFinding {
                        rule: AdmissionRule::InvalidFrontmatter,
                        severity: AdmissionSeverity::Quarantine,
                        detail: format!("could not inspect path: {error}"),
                    }],
                });
                return;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    findings.push(SkillDirFinding {
                        file: path.to_path_buf(),
                        findings: vec![AdmissionFinding {
                            rule: AdmissionRule::InvalidFrontmatter,
                            severity: AdmissionSeverity::Quarantine,
                            detail: format!("could not inspect directory entry: {error}"),
                        }],
                    });
                    continue;
                }
            };
            let entry_path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    findings.push(SkillDirFinding {
                        file: entry_path,
                        findings: vec![AdmissionFinding {
                            rule: AdmissionRule::InvalidFrontmatter,
                            severity: AdmissionSeverity::Quarantine,
                            detail: format!("could not inspect file type: {error}"),
                        }],
                    });
                    continue;
                }
            };
            if file_type.is_dir() {
                walk(&entry_path, findings);
            } else if file_type.is_file() {
                let bytes = match std::fs::read(&entry_path) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        findings.push(SkillDirFinding {
                            file: entry_path,
                            findings: vec![AdmissionFinding {
                                rule: AdmissionRule::InvalidFrontmatter,
                                severity: AdmissionSeverity::Quarantine,
                                detail: format!("could not read file: {error}"),
                            }],
                        });
                        continue;
                    }
                };
                let text = match String::from_utf8(bytes) {
                    Ok(text) => text,
                    Err(_) => continue,
                };
                if let AdmissionVerdict::Quarantined(file_findings) = scan_instruction_surface(&text) {
                    findings.push(SkillDirFinding { file: entry_path, findings: file_findings });
                }
            }
        }
    }

    let mut findings = Vec::new();
    walk(dir, &mut findings);
    findings.sort_by(|left, right| left.file.cmp(&right.file));
    findings
}

#[cfg(test)]
mod tests {
    use super::{admitted_skills, scan_skill_dir};
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
    fn scan_clean_directory_is_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("SKILL.md"), "---\nname: clean\n---\nplain").expect("write");
        assert!(scan_skill_dir(tmp.path()).is_empty());
    }

    #[test]
    fn scan_reports_nested_payload() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let nested = tmp.path().join("tests");
        std::fs::create_dir(&nested).expect("mkdir");
        let hidden = char::from_u32(0x200B).expect("zero-width character");
        std::fs::write(nested.join("fixture.txt"), format!("a{hidden}b")).expect("write");
        let findings = scan_skill_dir(tmp.path());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, nested.join("fixture.txt"));
    }

    #[test]
    fn scan_skips_non_utf8_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("asset.bin"), [0xff, 0xfe]).expect("write");
        assert!(scan_skill_dir(tmp.path()).is_empty());
    }

    #[test]
    fn scan_results_are_sorted() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hidden = char::from_u32(0x200B).expect("zero-width character");
        std::fs::write(tmp.path().join("z.txt"), format!("a{hidden}b")).expect("write");
        std::fs::write(tmp.path().join("a.txt"), format!("a{hidden}b")).expect("write");
        let findings = scan_skill_dir(tmp.path());
        assert_eq!(findings.iter().map(|finding| &finding.file).collect::<Vec<_>>(),
            vec![&tmp.path().join("a.txt"), &tmp.path().join("z.txt")]);
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
