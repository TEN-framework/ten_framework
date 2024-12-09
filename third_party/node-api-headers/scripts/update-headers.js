'use strict';

const { writeFile } = require('fs/promises');
const { Readable } = require('stream');
const { resolve } = require('path');
const { parseArgs } = require('util')
const { createInterface } = require('readline');
const { inspect } = require('util');
const { runClang } = require('../lib/clang-utils');
const { evaluate } = require('../lib/parse-utils');

/**
 * @returns {Promise<string>} Version string, eg. `'v19.6.0'`.
 */
async function getLatestReleaseVersion() {
    const response = await fetch('https://nodejs.org/download/release/index.json');
    const json = await response.json();
    return json[0].version;
}

/**
 * @param {NodeJS.ReadableStream} stream
 * @param {string} destination
 * @param {boolean} verbose
 * @returns {Promise<void>} The `writeFile` Promise.
 */
function removeExperimentals(stream, destination, verbose = false) {
    return new Promise((resolve, reject) => {
        const debug = (...args) => {
            if (verbose) {
                console.log(...args);
            }
        };
        const rl = createInterface(stream);

        /** @type {Array<'write' | 'ignore' | 'preprocessor'>} */
        const mode = ['write'];

        /** @type {Array<string>} */
        const preprocessor = [];

        /** @type {Array<string>} */
        const macroStack = [];

        /** @type {RegExpMatchArray | null} */
        let matches;

        let lineNumber = 0;
        let toWrite = '';

        const handlePreprocessor = (expression) => {
            const result = evaluate(expression);

            macroStack.push(expression);

            if (result === false) {
                debug(`Line ${lineNumber} Ignored '${expression}'`);
                mode.push('ignore');
                return false;
            } else {
                debug(`Line ${lineNumber} Pushed '${expression}'`);
                mode.push('write');
                return true;
            }
        };

        rl.on('line', function lineHandler(line) {
            ++lineNumber;
            if (matches = line.match(/^\s*#if(n)?def\s+([A-Za-z_][A-Za-z0-9_]*)/)) {
                const negated = Boolean(matches[1]);
                const identifier = matches[2];
                macroStack.push(identifier);

                debug(`Line ${lineNumber} Pushed ${identifier}`);

                if (identifier === 'NAPI_EXPERIMENTAL') {
                    if (negated) {
                        mode.push('write');
                    } else {
                        mode.push('ignore');
                    }
                    return;
                } else {
                    mode.push('write');
                }
            }
            else if (matches = line.match(/^\s*#if\s+(.+)$/)) {
                const expression = matches[1];
                if (expression.endsWith('\\')) {
                    if (preprocessor.length) {
                        reject(new Error(`Unexpected preprocessor continuation on line ${lineNumber}`));
                        return;
                    }
                    preprocessor.push(expression.substring(0, expression.length - 1));

                    mode.push('preprocessor');
                    return;
                } else {
                    if (!handlePreprocessor(expression)) {
                        return;
                    }
                }
            }
            else if (line.match(/^#else(?:\s+|$)/)) {
                const identifier = macroStack[macroStack.length - 1];

                debug(`Line ${lineNumber} Peeked ${identifier}`);

                if (!identifier) {
                    rl.off('line', lineHandler);
                    reject(new Error(`Macro stack is empty handling #else on line ${lineNumber}`));
                    return;
                }

                if (identifier.indexOf('NAPI_EXPERIMENTAL') > -1) {
                    const lastMode = mode[mode.length - 1];
                    mode[mode.length - 1] = (lastMode === 'ignore') ? 'write' : 'ignore';
                    return;
                }
            }
            else if (line.match(/^\s*#endif(?:\s+|$)/)) {
                const identifier = macroStack.pop();
                mode.pop();

                debug(`Line ${lineNumber} Popped ${identifier}`);

                if (!identifier) {
                    rl.off('line', lineHandler);
                    reject(new Error(`Macro stack is empty handling #endif on line ${lineNumber}`));
                    return;
                }

                if (identifier.indexOf('NAPI_EXPERIMENTAL') > -1) {
                    return;
                }
            }

            if (mode.length === 0) {
                rl.off('line', lineHandler);
                reject(new Error(`Write mode empty handling #endif on line ${lineNumber}`));
                return;
            }

            if (mode[mode.length - 1] === 'write') {
                toWrite += `${line}\n`;
            } else if (mode[mode.length - 1] === 'preprocessor') {
                if (!preprocessor) {
                    reject(new Error(`Preprocessor mode without preprocessor on line ${lineNumber}`));
                    return;
                }

                if (line.endsWith('\\')) {
                    preprocessor.push(line.substring(0, line.length - 1));
                    return;
                }

                preprocessor.push(line);

                const expression = preprocessor.join('');
                preprocessor.length = 0;
                mode.pop();

                if (!handlePreprocessor(expression)) {
                    return;
                }
            }

        });

        rl.on('close', () => {
            if (macroStack.length > 0) {
                reject(new Error(`Macro stack is not empty at EOF: ${inspect(macroStack)}`));
            }
            else if (mode.length > 1) {
                reject(new Error(`Write mode greater than 1 at EOF: ${inspect(mode)}`));
            }
            else if (toWrite.match(/^\s*#if(?:n)?def\s+NAPI_EXPERIMENTAL/m)) {
                reject(new Error(`Output has match for NAPI_EXPERIMENTAL`));
            }
            else {
                resolve(writeFile(destination, toWrite));
            }
        });
    });
}

/**
 * Validate syntax for a file using clang.
 * @param {string} path Path for file to validate with clang.
 */
async function validateSyntax(path) {
    try {
        await runClang(['-fsyntax-only', path]);
    } catch (e) {
        throw new Error(`Syntax validation failed for ${path}: ${e}`);
    }
}

async function main() {
    const { values: { tag, verbose } } = parseArgs({
        options: {
            tag: {
                type: 'string',
                short: 't',
                default: await getLatestReleaseVersion()
            },
            verbose: {
                type: 'boolean',
                short: 'v',
            },
        },
    });

    console.log(`feat: update headers from nodejs/node tag ${tag}`);

    const files = ['js_native_api_types.h', 'js_native_api.h', 'node_api_types.h', 'node_api.h'];

    for (const filename of files) {
        const url = `https://raw.githubusercontent.com/nodejs/node/${tag}/src/${filename}`;
        const path = resolve(__dirname, '..', 'include', filename);

        if (verbose) {
            console.log(`  ${url} -> ${path}`);
        }

        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`Fetch of ${url} returned ${response.status} ${response.statusText}`);
        }

        await removeExperimentals(Readable.fromWeb(response.body), path, verbose);

        await validateSyntax(path);
    }
}

main().catch(e => {
    console.error(e);
    process.exitCode = 1;
});
