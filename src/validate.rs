use crate::vocab::{IriStatus, VocabCache, load_vocabs};
use sophia::api::prelude::{Term, Triple, TripleSource};
use sophia::turtle::parser::{nt, turtle};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

struct Issues {
    invalid: HashSet<String>,
    https_warnings: HashSet<String>,
    unknown_vocab: HashSet<String>,
}

impl Issues {
    fn new() -> Self {
        Self {
            invalid: HashSet::new(),
            https_warnings: HashSet::new(),
            unknown_vocab: HashSet::new(),
        }
    }
}

/// Checks a single predicate (and optionally the rdf:type object) against the
/// vocab cache, pushing any findings into the appropriate issue bucket.
fn check_triple(p_str: &str, o_str: Option<&str>, vocabs: &VocabCache, issues: &mut Issues) {
    if p_str == RDF_TYPE {
        if let Some(obj) = o_str {
            match vocabs.check_class(obj) {
                IriStatus::Valid => {}
                IriStatus::HttpsSchemaOrg => { issues.https_warnings.insert(obj.to_string()); }
                IriStatus::Invalid => { issues.invalid.insert(format!("{obj} (class)")); }
                IriStatus::Unknown => { issues.unknown_vocab.insert(obj.to_string()); }
            }
        }
    } else {
        match vocabs.check_property(p_str) {
            IriStatus::Valid => {}
            IriStatus::HttpsSchemaOrg => { issues.https_warnings.insert(p_str.to_string()); }
            IriStatus::Invalid => { issues.invalid.insert(format!("{p_str} (property)")); }
            IriStatus::Unknown => { issues.unknown_vocab.insert(p_str.to_string()); }
        }
    }
}

pub fn run_validation(input: &str, format: Option<&str>) -> anyhow::Result<()> {
    let vocabs = load_vocabs()?;
    let mut issues = Issues::new();

    let buf_read: Box<dyn BufRead> = if input == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(input)?))
    };

    let fmt = format.unwrap_or_else(|| {
        if input.ends_with(".ttl") {
            "turtle"
        } else if input.ends_with(".nt") || input.ends_with(".nq") {
            "ntriples"
        } else {
            eprintln!("Warning: could not auto-detect format, defaulting to ntriples");
            "ntriples"
        }
    });

    if fmt == "turtle" || fmt == "ttl" {
        turtle::parse_bufread(buf_read)
            .for_each_triple(|t| {
                if let Some(p_iri) = t.p().iri() {
                    let p_str = p_iri.as_str();
                    let o_term = t.o();
                    let o_iri_opt = o_term.iri();
                    let o_str = o_iri_opt.as_ref().map(|iri| iri.as_str());
                    check_triple(p_str, o_str, &vocabs, &mut issues);
                }
            })
            .map_err(|e| anyhow::anyhow!("Turtle parse error: {e}"))?;
    } else {
        // N-Triples / N-Quads: parse line-by-line for error recovery
        for line_res in buf_read.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let res = nt::parse_str(trimmed).for_each_triple(|t| {
                if let Some(p_iri) = t.p().iri() {
                    let p_str = p_iri.as_str();
                    let o_term = t.o();
                    let o_iri_opt = o_term.iri();
                    let o_str = o_iri_opt.as_ref().map(|iri| iri.as_str());
                    check_triple(p_str, o_str, &vocabs, &mut issues);
                }
            });
            if let Err(e) = res {
                eprintln!("Warning: skipping invalid line: {trimmed} ({e})");
            }
        }
    }

    print_results(&issues);

    if !issues.invalid.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

fn print_sorted(items: &HashSet<String>) {
    let mut sorted: Vec<&String> = items.iter().collect();
    sorted.sort();
    for term in sorted {
        println!("  - {term}");
    }
}

fn print_results(issues: &Issues) {
    if !issues.invalid.is_empty() {
        println!("❌ Invalid terms ({}):", issues.invalid.len());
        print_sorted(&issues.invalid);
    }

    if !issues.https_warnings.is_empty() {
        println!("⚠️  HTTPS schema.org URLs ({}) — use http:// instead:", issues.https_warnings.len());
        print_sorted(&issues.https_warnings);
    }

    if !issues.unknown_vocab.is_empty() {
        println!("💡 Unknown vocabularies ({}) — not in cache:", issues.unknown_vocab.len());
        print_sorted(&issues.unknown_vocab);
    }

    if issues.invalid.is_empty() && issues.https_warnings.is_empty() && issues.unknown_vocab.is_empty() {
        println!("✅ All vocabulary usages are valid.");
    }
}
