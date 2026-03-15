# LLM Instructions for Validating Generated RDF

Whenever you (an LLM) generate or modify RDF data (in Turtle, N-Triples, N-Quads, etc.), you **MUST** ensure the validity of the vocabulary terms used (Classes and Properties) before presenting the final result.

To automatically validate your generated RDF dataset against all known vocabularies, invoke the `rvv` (RDF Vocab Validator) CLI tool. 

## Quickstart

**1. Fetch the latest vocabularies (if caching for the first time or updating):**
```bash
rvv update-vocabs
```
*Note: This downloads a highly compacted map of all recognized terms from `@zazuko/rdf-vocabularies` into local storage to provide sub-millisecond validations.*

**2. Validate a file:**
```bash
rvv validate path/to/your_generated.ttl
```
or auto-detect stream:
```bash
cat path/to/your_generated.nq | rvv validate -
```

## How It Works

- The validator parses streams and strictly checks `rdf:type` objects (for classes) and properties against the known Zazuko vocabularies cache. (The `https://schema.org/` prefix receives a special waiver).
- **Format Support:** N-Triples and Turtle are auto-detected by extension (`.nt`, `.nq`, `.ttl`) or fallback to N-Triples streaming parsing.
- **Error Recovery:** By default, for N-Triples/N-Quads, `rvv` continues parsing if one line has bad syntax or invalid vocabulary, printing the errors to `stderr` and the line details to allow your agents to fix specific lines without losing context of the rest of the file.

## Expected Outcomes

If no errors are found, the tool exits with `0` successfully:
```text
✅ All vocabulary usages are valid.
```

If it fails, it exits with `1` and prints messages like:
```text
❌ Invalid terms (2):
  - http://xmlns.com/foaf/0.1/names (property)
  - http://schema.org/NonExistentClass (class)
```

Other non-fatal output (exit code `0`):
```text
⚠️  HTTPS schema.org URLs (1) — use http:// instead:
  - https://schema.org/Person

💡 Unknown vocabularies (1) — not in cache:
  - http://example.org/custom/MyThing
```

**Your Action:** If you see `❌ Invalid terms`, you generated an IRI that does not exist in the official ontological definitions. Rewrite the RDF using correct terms (e.g., changing `foaf:names` to `foaf:name`) and validate again before delivering the response.
