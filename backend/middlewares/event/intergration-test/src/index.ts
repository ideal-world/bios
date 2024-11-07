import { createTopic } from './api';
import tests from './tests'

const args = process.argv.slice(2);
const subcommand = args[0];

switch (subcommand) {
    case 'test':
        {
            const testCase = args[1];
            await subcommandTest(testCase);
        }
        break;
    case 'topic':
        {
            const verb = args[1];
            const topic = args[2];
            const options = args.slice(3).reduce<Record<string, unknown>>((options, option, _idx, _arr) => {
                const [k, v] = option.split('=');
                if (k.startsWith('--')) {
                    const key = k.slice(2);
                    const value = v ? JSON.parse(v) : true;
                    options[key] = value;
                }
                return options;
            }, {} as Record<string, unknown>);
            await subcommandTopic(verb, topic, options);
        }
        break;
    case 'help':
    default:
        console.log('Usage: node index.js test');
}


async function subcommandTopic(verb: string, topic: string, options: {
    blocking?: boolean
}) {
    switch (verb) {
        case 'create':
            await createTopic(topic, options.blocking);
            break;
        default:
            throw new Error(`Unrecognized verb: ${verb}`);
    }
}
async function subcommandTest(testCase: string) {
    if (Object.hasOwn(tests, testCase)) {
        const key = testCase as keyof typeof tests;
        await (tests[key])();
        console.info(`Test case ${testCase} finished`);
    } else {
        console.info(`Available test cases: `);
        for (const key in tests) {
            console.info(`  ${key}`);
        }
        throw new Error(`Test case not found: ${testCase}`);
    }
}
