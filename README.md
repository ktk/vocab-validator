# RDF Vocabulary Validator

A 100% vibe coded CLI tool to validate if RDF files contain valid classes and properties by checking them against standard vocabularies.

## Features

- **Class & Property Validation**: Uses `@zazuko/rdf-vocabularies` to verify that terms are defined in their respective ontologies.
- **HTTPS Schema.org Warning**: Automatically flags `https://schema.org/` IRIs as warnings (recommending `http://schema.org/` for consistency).
- **Categorized Reporting**: Distinguishes between definitely invalid terms, unknown vocabularies, and warnings.
- **Modular Design**: Built on top of `@zazuko/env-node`, making the core logic reusable for IDE integrations.

## Installation

1. Clone the repository.
2. Install dependencies:
   ```bash
   npm install
   ```

## Usage

Run the validator against a Turtle (`.ttl`) file:

```bash
npm run validate -- path/to/your/file.ttl
```

### Exit Codes
- `0`: Valid (or only warnings/unknown vocabularies found).
- `1`: Invalid terms found (missing from their vocabulary).

## Development

- `src/validator.ts`: Main validation logic.
- `src/vocabularies.ts`: Vocabulary lookup utility.
- `src/index.ts`: CLI interface.

### Running Samples

Several sample files are provided in the `samples/` directory to demonstrate different scenarios:

```bash
# Valid file
npm run validate -- samples/valid.ttl

# File with invalid schema.org terms
npm run validate -- samples/invalid.ttl

# File with https://schema.org/ warnings
npm run validate -- samples/https_schema.ttl

# File with unknown custom vocabularies
npm run validate -- samples/unknown.ttl
```

## Built With

- [@zazuko/env](https://github.com/zazuko/env)
- [@zazuko/rdf-vocabularies](https://github.com/zazuko/rdf-vocabularies)
- [Commander.js](https://github.com/tj/commander.js)

