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
            await subcommandTopic(verb, topic);
        }
        break;
    case 'help':
    default:
        console.log('Usage: node index.js test');
}


async function subcommandTopic(verb: string, topic: string) {
    switch (verb) {
        case 'create':
            await createTopic(topic);
            break;
        default:
            throw new Error(`Unrecognized verb: ${verb}`);
    }
}
async function subcommandTest(testCase: string) {
    if (Object.hasOwn(tests, testCase)) {
        const key = testCase as keyof typeof tests;
        await (tests[key])();
    } else {
        console.info(`Available test cases: `);
        for (const key in tests) {
            console.info(`  ${key}`);
        }
        throw new Error(`Test case not found: ${testCase}`);
    }
}
