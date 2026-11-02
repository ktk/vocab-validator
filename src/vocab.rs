use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sophia::api::prelude::{Quad, QuadSource, Term};
use sophia::turtle::parser::nq;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use tar::Archive;

/// Result of checking a single IRI against the vocabulary cache.
pub enum IriStatus {
    /// Term found in the cache.
    Valid,
    /// Namespace is known but the specific term was not found.
    Invalid,
    /// IRI uses `https://schema.org/` instead of `http://schema.org/`.
    HttpsSchemaOrg,
    /// Namespace is not present in the vocabulary cache at all.
    Unknown,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct VocabCache {
    pub classes: HashSet<String>,
    pub properties: HashSet<String>,
    /// All known namespace prefixes (IRI up to and including the last `#` or `/`).
    pub namespaces: HashSet<String>,
}

impl VocabCache {
    pub fn check_class(&self, iri: &str) -> IriStatus {
        if iri.starts_with("https://schema.org/") {
            return IriStatus::HttpsSchemaOrg;
        }
        if self.classes.contains(iri) {
            IriStatus::Valid
        } else if self.namespaces.contains(namespace_of(iri)) {
            IriStatus::Invalid
        } else {
            IriStatus::Unknown
        }
    }

    pub fn check_property(&self, iri: &str) -> IriStatus {
        if iri.starts_with("https://schema.org/") {
            return IriStatus::HttpsSchemaOrg;
        }
        if self.properties.contains(iri) {
            IriStatus::Valid
        } else if self.namespaces.contains(namespace_of(iri)) {
            IriStatus::Invalid
        } else {
            IriStatus::Unknown
        }
    }
}

/// Extracts the namespace prefix of an IRI (everything up to and including
/// the last `#` or `/`).
fn namespace_of(iri: &str) -> &str {
    iri.rfind(['#', '/'])
        .map(|pos| &iri[..=pos])
        .unwrap_or(iri)
}

pub fn update_vocabs() -> Result<()> {
    eprintln!("Fetching metadata for @zazuko/rdf-vocabularies...");
    let client = Client::new();
    let resp: serde_json::Value = client
        .get("https://registry.npmjs.org/@zazuko/rdf-vocabularies")
        .send()?
        .json()?;

    let latest_version = resp["dist-tags"]["latest"]
        .as_str()
        .context("Could not find latest version tag")?;
    let tarball_url = resp["versions"][latest_version]["dist"]["tarball"]
        .as_str()
        .context("Could not find tarball URL")?;

    eprintln!("Downloading tarball for v{latest_version}...");
    let tarball_resp = client.get(tarball_url).send()?;

    eprintln!("Extracting and parsing vocabularies...");
    let tar = GzDecoder::new(tarball_resp);
    let mut archive = Archive::new(tar);

    let mut cache = VocabCache::default();
    let mut file_count = 0;

    let class_types: HashSet<&str> = [
        "http://www.w3.org/2000/01/rdf-schema#Class",
        "http://www.w3.org/2002/07/owl#Class",
    ]
    .into_iter()
    .collect();

    let property_types: HashSet<&str> = [
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property",
        "http://www.w3.org/2002/07/owl#DatatypeProperty",
        "http://www.w3.org/2002/07/owl#ObjectProperty",
        "http://www.w3.org/2002/07/owl#AnnotationProperty",
        // Property characteristics: vocabularies often type a term only with one of these
        // (e.g. void:class is just an owl:FunctionalProperty), but each implies a property.
        "http://www.w3.org/2002/07/owl#FunctionalProperty",
        "http://www.w3.org/2002/07/owl#InverseFunctionalProperty",
        "http://www.w3.org/2002/07/owl#TransitiveProperty",
        "http://www.w3.org/2002/07/owl#SymmetricProperty",
        "http://www.w3.org/2002/07/owl#AsymmetricProperty",
        "http://www.w3.org/2002/07/owl#ReflexiveProperty",
        "http://www.w3.org/2002/07/owl#IrreflexiveProperty",
        "http://www.w3.org/2002/07/owl#DeprecatedProperty",
    ]
    .into_iter()
    .collect();

    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();

        if path.starts_with("package/ontologies/") && path.ends_with(".nq") {
            let buf_reader = BufReader::new(entry);
            file_count += 1;
            nq::parse_bufread(buf_reader)
                .for_each_quad(|q| {
                    if let Some(p) = q.p().iri() {
                        if p.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                            if let (Some(s), Some(o)) = (q.s().iri(), q.o().iri()) {
                                let s_str = s.as_str();
                                let o_str = o.as_str();
                                if class_types.contains(o_str) {
                                    cache.namespaces.insert(namespace_of(s_str).to_string());
                                    cache.classes.insert(s_str.to_string());
                                } else if property_types.contains(o_str) {
                                    cache.namespaces.insert(namespace_of(s_str).to_string());
                                    cache.properties.insert(s_str.to_string());
                                }
                            }
                        }
                    }
                })
                .ok();
        }
    }

    eprintln!(
        "Parsed {file_count} files: {} classes, {} properties across {} namespaces.",
        cache.classes.len(),
        cache.properties.len(),
        cache.namespaces.len(),
    );

    let cache_path = get_cache_path()?;
    eprintln!("Saving cache to {}", cache_path.display());
    let encoded: Vec<u8> = bincode::serialize(&cache)?;
    let mut file = File::create(cache_path)?;
    file.write_all(&encoded)?;

    Ok(())
}

pub fn load_vocabs() -> Result<VocabCache> {
    let cache_path = get_cache_path()?;
    if !cache_path.exists() {
        eprintln!("Warning: vocabulary cache not found. Run `rvv update-vocabs` first.");
        return Ok(VocabCache::default());
    }
    let file = File::open(cache_path)?;
    let cache: VocabCache = bincode::deserialize_from(file)?;
    Ok(cache)
}

fn get_cache_path() -> Result<PathBuf> {
    let mut path = dirs::cache_dir().context("No cache dir found")?;
    path.push("rvv");
    std::fs::create_dir_all(&path)?;
    path.push("vocabs.bin");
    Ok(path)
}
