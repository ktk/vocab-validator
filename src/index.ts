import { Command } from 'commander';
import { Validator } from './validator.js';
import fs from 'fs';
import path from 'path';

const program = new Command();

program
    .name('rdf-vocab-validator')
    .description('Validate RDF vocabularies (classes and properties)')
    .version('1.0.0');

program
    .argument('<file>', 'RDF file to validate')
    .action(async (file) => {
        const filePath = path.resolve(file);
        if (!fs.existsSync(filePath)) {
            console.error(`File not found: ${filePath}`);
            process.exit(1);
        }

        const inputStream = fs.createReadStream(filePath);
        const validator = new Validator();

        try {
            console.log(`Validating ${file}...`);
            const results = await validator.validate(inputStream);

            if (results.length === 0) {
                console.log('✅ No issues found.');
            } else {
                console.log(`\nFound ${results.length} issues:\n`);

                const invalids = results.filter(r => r.status === 'invalid');
                const unknowns = results.filter(r => r.status === 'unknown_vocabulary');
                const httpsSchema = results.filter(r => r.status === 'https_schema_org');

                if (invalids.length > 0) {
                    console.log('❌ Invalid terms (not found in their vocabularies):');
                    invalids.forEach(r => {
                        console.log(`  - [${r.type}] ${r.term}`);
                    });
                    console.log();
                }

                if (httpsSchema.length > 0) {
                    console.log('⚠️ HTTPS schema.org URLs (common but can lead to issues, use http instead):');
                    httpsSchema.forEach(r => {
                        console.log(`  - [${r.type}] ${r.term}`);
                    });
                    console.log();
                }

                if (unknowns.length > 0) {
                    console.log('💡 Unknown vocabularies (could not be verified):');
                    unknowns.forEach(r => {
                        console.log(`  - [${r.type}] ${r.term}`);
                    });
                    console.log();
                }

                if (invalids.length > 0) {
                    process.exit(1);
                }
            }
        } catch (error: any) {
            console.error(`Error during validation: ${error.message}`);
            process.exit(1);
        }
    });

program.parse();
