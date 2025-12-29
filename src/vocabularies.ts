import env from '@zazuko/env-node';
import { expandWithCheck } from '@zazuko/rdf-vocabularies/expandWithCheck';
import { shrink } from '@zazuko/rdf-vocabularies/shrink';

export const CLASS_TYPES = [
    'http://www.w3.org/2000/01/rdf-schema#Class',
    'http://www.w3.org/2002/07/owl#Class'
];

export const PROPERTY_TYPES = [
    'http://www.w3.org/1999/02/22-rdf-syntax-ns#Property',
    'http://www.w3.org/2002/07/owl#DatatypeProperty',
    'http://www.w3.org/2002/07/owl#ObjectProperty',
    'http://www.w3.org/2002/07/owl#AnnotationProperty'
];

export type ValidationStatus = 'valid' | 'invalid' | 'unknown_vocabulary' | 'https_schema_org';

export async function validateTerm(termIri: string, types: string[]): Promise<ValidationStatus> {
    const prefixed = shrink(termIri);
    if (!prefixed) {
        return 'unknown_vocabulary';
    }

    const result = await expandWithCheck(prefixed, types);
    return result ? 'valid' : 'invalid';
}

export async function isClass(termIri: string): Promise<ValidationStatus> {
    return validateTerm(termIri, CLASS_TYPES);
}

export async function isProperty(termIri: string): Promise<ValidationStatus> {
    return validateTerm(termIri, PROPERTY_TYPES);
}
