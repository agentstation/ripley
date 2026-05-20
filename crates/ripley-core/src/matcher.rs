use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::feed::Advisory;
use crate::lockfile::InstalledPackage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub advisory: Advisory,
    pub package: InstalledPackage,
    pub project_path: PathBuf,
}

pub fn find_matches(
    advisories: &[Advisory],
    packages: &[InstalledPackage],
    project_path: &Path,
) -> Vec<Match> {
    let mut matches = Vec::new();

    for advisory in advisories {
        for pkg in packages {
            if pkg.ecosystem != advisory.ecosystem || pkg.name != advisory.package {
                continue;
            }

            let is_affected = advisory.affected_ranges.iter().any(|range| {
                let after_introduced = pkg.version >= range.introduced;
                let before_fixed = match &range.fixed {
                    Some(fixed) => pkg.version < *fixed,
                    None => true,
                };
                after_introduced && before_fixed
            });

            if is_affected {
                matches.push(Match {
                    advisory: advisory.clone(),
                    package: pkg.clone(),
                    project_path: project_path.to_path_buf(),
                });
            }
        }
    }

    matches.sort_by(|a, b| {
        let sev_a = a.advisory.severity.as_ref();
        let sev_b = b.advisory.severity.as_ref();
        sev_b
            .cmp(&sev_a)
            .then_with(|| a.package.name.cmp(&b.package.name))
    });

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::AffectedRange;
    use crate::types::{Ecosystem, Severity};

    fn advisory(package: &str, introduced: &str, fixed: Option<&str>) -> Advisory {
        Advisory {
            id: format!("GHSA-test-{package}"),
            ecosystem: Ecosystem::Npm,
            package: package.to_string(),
            affected_ranges: vec![AffectedRange {
                introduced: semver::Version::parse(introduced).expect("parse"),
                fixed: fixed.map(|v| semver::Version::parse(v).expect("parse")),
            }],
            severity: Some(Severity::High),
            summary: format!("Vulnerability in {package}"),
            references: Vec::new(),
        }
    }

    fn package(name: &str, version: &str) -> InstalledPackage {
        InstalledPackage {
            name: name.to_string(),
            version: semver::Version::parse(version).expect("parse"),
            ecosystem: Ecosystem::Npm,
        }
    }

    #[test]
    fn test_match_found() {
        let advisories = vec![advisory("lodash", "0.0.0", Some("4.17.21"))];
        let packages = vec![package("lodash", "4.17.20")];
        let matches = find_matches(&advisories, &packages, Path::new("/test"));

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].advisory.id, "GHSA-test-lodash");
        assert_eq!(matches[0].package.name, "lodash");
    }

    #[test]
    fn test_no_match() {
        let advisories = vec![advisory("lodash", "0.0.0", Some("4.17.21"))];
        let packages = vec![package("lodash", "4.17.21")];
        let matches = find_matches(&advisories, &packages, Path::new("/test"));

        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_fixed_version() {
        let advisories = vec![advisory("bad-pkg", "2.0.0", None)];
        let packages = vec![package("bad-pkg", "3.5.0")];
        let matches = find_matches(&advisories, &packages, Path::new("/test"));

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_multiple_ranges() {
        let adv = Advisory {
            id: "GHSA-multi".to_string(),
            ecosystem: Ecosystem::Npm,
            package: "multi-pkg".to_string(),
            affected_ranges: vec![
                AffectedRange {
                    introduced: semver::Version::new(1, 0, 0),
                    fixed: Some(semver::Version::new(1, 0, 5)),
                },
                AffectedRange {
                    introduced: semver::Version::new(2, 0, 0),
                    fixed: Some(semver::Version::new(2, 0, 3)),
                },
            ],
            severity: Some(Severity::Critical),
            summary: "Multiple ranges".to_string(),
            references: Vec::new(),
        };

        let safe = package("multi-pkg", "1.0.5");
        assert!(find_matches(&[adv.clone()], &[safe], Path::new("/test")).is_empty());

        let affected_range1 = package("multi-pkg", "1.0.3");
        assert_eq!(
            find_matches(&[adv.clone()], &[affected_range1], Path::new("/test")).len(),
            1
        );

        let affected_range2 = package("multi-pkg", "2.0.1");
        assert_eq!(
            find_matches(&[adv], &[affected_range2], Path::new("/test")).len(),
            1
        );
    }

    #[test]
    fn test_different_ecosystem_no_match() {
        let advisories = vec![advisory("lodash", "0.0.0", Some("4.17.21"))];
        let packages = vec![InstalledPackage {
            name: "lodash".to_string(),
            version: semver::Version::new(4, 17, 20),
            ecosystem: Ecosystem::PyPI,
        }];
        let matches = find_matches(&advisories, &packages, Path::new("/test"));

        assert!(matches.is_empty());
    }

    #[test]
    fn test_sort_by_severity() {
        let mut adv_low = advisory("pkg-a", "0.0.0", None);
        adv_low.severity = Some(Severity::Low);

        let mut adv_critical = advisory("pkg-b", "0.0.0", None);
        adv_critical.severity = Some(Severity::Critical);

        let packages = vec![package("pkg-a", "1.0.0"), package("pkg-b", "1.0.0")];
        let matches = find_matches(&[adv_low, adv_critical], &packages, Path::new("/test"));

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].advisory.severity, Some(Severity::Critical));
        assert_eq!(matches[1].advisory.severity, Some(Severity::Low));
    }
}
