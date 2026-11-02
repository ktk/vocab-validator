# rvv — RDF Vocabulary Validator

A fast CLI tool that validates RDF files against standard ontologies. It checks every class and property IRI used in a file against a local cache of well-known vocabularies (schema.org, FOAF, RDF, RDFS, OWL, and [80+ more](https://github.com/zazuko/rdf-vocabularies)).

## Install

### Cargo

```bash
cargo install zazuko-rvv
```

The crate is called `zazuko-rvv`; the installed command is `rvv`.

### Homebrew

```bash
brew tap ktk/tap
brew install rvv
```

### Build from source

Requires Rust 2024 edition (rustup recommended).

```bash
git clone https://github.com/ktk/vocab-validator
cd vocab-validator
cargo build --release
# binary at: target/release/rvv
```

After installing, fetch the vocabulary cache:

```bash
rvv update-vocabs
```

## Usage

```bash
rvv validate path/to/file.ttl
rvv validate path/to/file.nt
rvv validate -          # read from stdin
```

The `--format` flag overrides auto-detection from extension (`turtle` or `ntriples`):

```bash
rvv validate --format turtle data.rdf
```

### Output

Results are grouped into three categories:

```
❌ Invalid terms (1):
  - http://schema.org/naem (property)

⚠️  HTTPS schema.org URLs (1):
  - https://schema.org/Person

💡 Unknown vocabularies (1):
  - http://example.org/custom/MyClass
```

- **Invalid** — the namespace is known but the term doesn't exist. Exit code `1`.
- **HTTPS schema.org** — using `https://schema.org/` instead of `http://schema.org/`. Exit code `0`.
- **Unknown** — the namespace isn't in the vocabulary cache at all. Exit code `0`.

If everything is valid: `✅ All vocabulary usages are valid.`

N-Triples and N-Quads are parsed line-by-line, so a single malformed line is skipped with a warning rather than aborting the whole file.

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | Valid (warnings and unknown vocabularies don't fail) |
| `1` | One or more invalid terms found |

## Vocabulary cache

The cache is built from the [`@zazuko/rdf-vocabularies`](https://github.com/zazuko/rdf-vocabularies) NPM package and stored in the platform cache directory (`~/Library/Caches/rvv/` on macOS, `~/.cache/rvv/` on Linux).

Refresh it any time to pick up new vocabulary versions:

```bash
rvv update-vocabs
```

## Samples

```bash
rvv validate samples/valid.ttl        # ✅ all valid
rvv validate samples/invalid.ttl      # ❌ invalid terms
rvv validate samples/https_schema.ttl # ⚠️  https warnings
rvv validate samples/unknown.ttl      # 💡 unknown vocabularies
rvv validate samples/error_recovery.nt # line-by-line error recovery demo
```

## Built with

- [sophia](https://github.com/pchampin/sophia_rs) — RDF parsing
- [@zazuko/rdf-vocabularies](https://github.com/zazuko/rdf-vocabularies) — ontology data
- [clap](https://github.com/clap-rs/clap) — CLI parsing
