use serde::Serialize;

use crate::types::Ecosystem;

#[derive(Debug, Clone, Serialize)]
pub struct TyposquatMatch {
    pub input: String,
    pub canonical: String,
    pub score: f64,
    pub reason: String,
}

pub fn check_typosquat(name: &str, ecosystem: Ecosystem) -> Option<TyposquatMatch> {
    let popular = popular_packages(ecosystem);
    if popular.is_empty() {
        return None;
    }

    // Already a known popular package — not a typosquat
    if popular.contains(&name) {
        return None;
    }

    let normalized = normalize_homoglyphs(name);
    let check_name = if normalized != name {
        &normalized
    } else {
        name
    };

    if normalized != name && popular.contains(&normalized.as_str()) {
        return Some(TyposquatMatch {
            input: name.to_string(),
            canonical: normalized.clone(),
            score: 1.0,
            reason: "Unicode homoglyph of popular package".to_string(),
        });
    }

    let mut best: Option<TyposquatMatch> = None;

    for &popular_name in popular {
        let distance = weighted_damerau_levenshtein(check_name, popular_name);

        // Only flag if edit distance is small relative to name length
        let max_name_len = check_name.len().max(popular_name.len());
        if max_name_len == 0 {
            continue;
        }

        let threshold = match max_name_len {
            0..=3 => 1,
            4..=7 => 1,
            8..=12 => 2,
            _ => 3,
        };

        if distance > 0 && distance <= threshold {
            let mut score = 1.0 - (distance as f64 / max_name_len as f64);

            // Boost score when differing characters are keyboard-adjacent (likely typos)
            let adjacency = keyboard_adjacency_ratio(check_name, popular_name);
            score += adjacency * 0.1;

            if best.as_ref().is_none_or(|b| score > b.score) {
                best = Some(TyposquatMatch {
                    input: name.to_string(),
                    canonical: popular_name.to_string(),
                    score,
                    reason: format!(
                        "edit distance {distance} from popular package \"{popular_name}\""
                    ),
                });
            }
        }
    }

    best
}

fn weighted_damerau_levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut d = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in d.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, val) in d[0].iter_mut().enumerate().take(b_len + 1) {
        *val = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);

            // Transposition
            if i > 1
                && j > 1
                && a_chars[i - 1] == b_chars[j - 2]
                && a_chars[i - 2] == b_chars[j - 1]
            {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }

    d[a_len][b_len]
}

fn are_keyboard_adjacent(a: char, b: char) -> bool {
    let a_lower = a.to_ascii_lowercase();
    let b_lower = b.to_ascii_lowercase();

    let neighbors = match a_lower {
        'q' => "wa",
        'w' => "qeas",
        'e' => "wrds",
        'r' => "etfd",
        't' => "rygf",
        'y' => "tuhg",
        'u' => "yijh",
        'i' => "uokj",
        'o' => "iplk",
        'p' => "ol",
        'a' => "qwsz",
        's' => "awedxz",
        'd' => "serfcx",
        'f' => "drtgvc",
        'g' => "ftyhbv",
        'h' => "gyujnb",
        'j' => "huikmn",
        'k' => "jiolm",
        'l' => "kop",
        'z' => "asx",
        'x' => "zsdc",
        'c' => "xdfv",
        'v' => "cfgb",
        'b' => "vghn",
        'n' => "bhjm",
        'm' => "njk",
        _ => "",
    };

    neighbors.contains(b_lower)
}

fn keyboard_adjacency_ratio(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let min_len = a_chars.len().min(b_chars.len());
    if min_len == 0 {
        return 0.0;
    }
    let mut diffs = 0;
    let mut adjacent = 0;
    for i in 0..min_len {
        if a_chars[i] != b_chars[i] {
            diffs += 1;
            if are_keyboard_adjacent(a_chars[i], b_chars[i]) {
                adjacent += 1;
            }
        }
    }
    if diffs == 0 {
        return 0.0;
    }
    adjacent as f64 / diffs as f64
}

fn normalize_homoglyphs(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Greek/Cyrillic homoglyphs for Latin letters
            '\u{03BF}' => 'o', // Greek omicron → o
            '\u{043E}' => 'o', // Cyrillic o → o
            '\u{0430}' => 'a', // Cyrillic a → a
            '\u{03B1}' => 'a', // Greek alpha → a
            '\u{0435}' => 'e', // Cyrillic e → e
            '\u{03B5}' => 'e', // Greek epsilon → e
            '\u{0456}' => 'i', // Cyrillic i → i
            '\u{0441}' => 'c', // Cyrillic s (looks like c) → c
            '\u{0440}' => 'p', // Cyrillic r (looks like p) → p
            '\u{0445}' => 'x', // Cyrillic kha (looks like x) → x
            '\u{0443}' => 'y', // Cyrillic u (looks like y) → y
            '\u{0455}' => 's', // Cyrillic dze → s
            '\u{04BB}' => 'h', // Cyrillic shha → h
            '\u{0501}' => 'd', // Cyrillic komi de → d
            '\u{0261}' => 'g', // Latin small script g → g
            '\u{01C0}' => 'l', // Latin dental click (looks like l) → l
            '\u{FF4F}' => 'o', // Fullwidth o → o
            '\u{FF41}' => 'a', // Fullwidth a → a
            '\u{FF45}' => 'e', // Fullwidth e → e
            _ => c,
        })
        .collect()
}

fn popular_packages(ecosystem: Ecosystem) -> &'static [&'static str] {
    match ecosystem {
        Ecosystem::Npm => NPM_TOP_PACKAGES,
        _ => &[],
    }
}

const NPM_TOP_PACKAGES: &[&str] = &[
    "express",
    "lodash",
    "react",
    "react-dom",
    "axios",
    "chalk",
    "commander",
    "debug",
    "dotenv",
    "eslint",
    "glob",
    "inquirer",
    "jest",
    "jquery",
    "minimist",
    "mkdirp",
    "moment",
    "mongodb",
    "mongoose",
    "next",
    "node-fetch",
    "nodemon",
    "npm",
    "passport",
    "pg",
    "prettier",
    "prop-types",
    "puppeteer",
    "redis",
    "request",
    "rimraf",
    "rxjs",
    "semver",
    "socket.io",
    "typescript",
    "underscore",
    "uuid",
    "webpack",
    "winston",
    "ws",
    "yargs",
    "async",
    "bluebird",
    "body-parser",
    "cheerio",
    "classnames",
    "colors",
    "cookie-parser",
    "cors",
    "cross-env",
    "css-loader",
    "dayjs",
    "ejs",
    "ember-cli",
    "enzyme",
    "execa",
    "fastify",
    "formik",
    "fs-extra",
    "got",
    "graphql",
    "gulp",
    "handlebars",
    "helmet",
    "http-proxy",
    "immer",
    "jsdom",
    "json5",
    "jsonwebtoken",
    "koa",
    "less",
    "lerna",
    "lru-cache",
    "marked",
    "material-ui",
    "meow",
    "micro",
    "mime",
    "mocha",
    "morgan",
    "multer",
    "mysql",
    "nan",
    "nock",
    "node-gyp",
    "node-sass",
    "nunjucks",
    "nyc",
    "ora",
    "p-limit",
    "pino",
    "pm2",
    "postcss",
    "pug",
    "qs",
    "ramda",
    "rollup",
    "sass",
    "sequelize",
    "sharp",
    "shelljs",
    "sinon",
    "slugify",
    "sql.js",
    "styled-components",
    "supertest",
    "svelte",
    "tailwindcss",
    "tape",
    "through2",
    "tslib",
    "validator",
    "vite",
    "vue",
    "webpack-cli",
    "xml2js",
    "yaml",
    "zod",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typosquat_detection() {
        // "expresss" is 1 edit from "express"
        let result = check_typosquat("expresss", Ecosystem::Npm);
        assert!(result.is_some(), "should detect 'expresss' as typosquat");
        let m = result.expect("checked above");
        assert_eq!(m.canonical, "express");
        assert!(m.score > 0.5);
    }

    #[test]
    fn test_typosquat_transposition() {
        // "exrpess" has transposed 'r' and 'p'
        let result = check_typosquat("exrpess", Ecosystem::Npm);
        assert!(result.is_some(), "should detect transposition typosquat");
        let m = result.expect("checked above");
        assert_eq!(m.canonical, "express");
    }

    #[test]
    fn test_homoglyph() {
        // "lοdash" with Greek omicron (ο) instead of Latin 'o'
        let result = check_typosquat("l\u{03BF}dash", Ecosystem::Npm);
        assert!(result.is_some(), "should detect homoglyph");
        let m = result.expect("checked above");
        assert_eq!(m.canonical, "lodash");
        assert!(m.reason.contains("homoglyph"));
    }

    #[test]
    fn test_exact_match_not_flagged() {
        let result = check_typosquat("express", Ecosystem::Npm);
        assert!(result.is_none(), "exact match should not be flagged");
    }

    #[test]
    fn test_unrelated_name_not_flagged() {
        let result = check_typosquat("my-totally-unique-package-name", Ecosystem::Npm);
        assert!(result.is_none(), "unrelated name should not be flagged");
    }

    #[test]
    fn test_unknown_ecosystem_returns_none() {
        let result = check_typosquat("expresss", Ecosystem::Go);
        assert!(
            result.is_none(),
            "should return None for ecosystem with no popular list"
        );
    }

    #[test]
    fn test_keyboard_adjacency() {
        assert!(are_keyboard_adjacent('e', 'r'));
        assert!(are_keyboard_adjacent('r', 'e'));
        assert!(!are_keyboard_adjacent('a', 'l'));
    }
}
