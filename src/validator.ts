import env from '@zazuko/env-node';
import { isClass, isProperty } from './vocabularies.js';
import type { ValidationStatus } from './vocabularies.js';
import { Stream } from 'stream';

export interface ValidationResult {
    term: string;
    type: 'class' | 'property';
    status: ValidationStatus;
    message?: string;
}

export class Validator {
    async validate(inputStream: Stream): Promise<ValidationResult[]> {
        const results: ValidationResult[] = [];
        const dataset = env.dataset();

        const parser = env.formats.parsers.get('text/turtle');
        if (!parser) {
            throw new Error('No turtle parser found');
        }

        const quadStream = parser.import(inputStream);
        await dataset.import(quadStream as any);

        // Use sets to avoid duplicate checks
        const propertiesToCheck = new Set<string>();
        const classesToCheck = new Set<string>();

        for (const quad of dataset) {
            if (quad.predicate.termType === 'NamedNode') {
                propertiesToCheck.add(quad.predicate.value);
            }

            if (quad.predicate.value === 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type' && quad.object.termType === 'NamedNode') {
                classesToCheck.add(quad.object.value);
            }
        }

        for (const prop of propertiesToCheck) {
            if (prop.startsWith('https://schema.org/')) {
                results.push({
                    term: prop,
                    type: 'property',
                    status: 'https_schema_org',
                });
                continue;
            }
            const status = await isProperty(prop);
            if (status !== 'valid') {
                results.push({
                    term: prop,
                    type: 'property',
                    status: status,
                });
            }
        }

        for (const cls of classesToCheck) {
            if (cls.startsWith('https://schema.org/')) {
                results.push({
                    term: cls,
                    type: 'class',
                    status: 'https_schema_org',
                });
                continue;
            }
            const status = await isClass(cls);
            if (status !== 'valid') {
                results.push({
                    term: cls,
                    type: 'class',
                    status: status,
                });
            }
        }

        return results;
    }
}
