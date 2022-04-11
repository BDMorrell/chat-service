import { opendir, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { compileFromFile } from 'json-schema-to-typescript';

const SCHEMA_DIRECTORY = "./json-schemas";
const OUTPUT_DIRECTORY = "./out";

async function main() {
    await mkdir(OUTPUT_DIRECTORY).catch((e) => {
        if (e.code !== "EEXIST") {
            throw e;
        }
    });
    const directory = await opendir(SCHEMA_DIRECTORY);
    for await (const file of directory) {
        const potential_ending = '.json';
        if (!file.name.endsWith(potential_ending) || file.name[0] === '.' || !file.isFile()) {
            continue;
        }
        const basename = file.name.substring(0, file.name.length - potential_ending.length);
        const output_file_name = path.join(OUTPUT_DIRECTORY, basename + '.d.ts');
        console.info("output_file_name: " + output_file_name);
        const promise = compileFromFile(path.join(SCHEMA_DIRECTORY, file.name))
            .then((ts_source) => writeFile(output_file_name, ts_source));
        await promise;
    }
}

main()
