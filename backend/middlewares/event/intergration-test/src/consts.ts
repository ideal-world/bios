const SERVER_URI = process.env['SERVER_URI'] ?? 'http://localhost:3000/event';
const TARDIS_CTX = process.env['TARDIS_CTX'] ?? btoa(JSON.stringify({
    "owner": "",
    "own_paths": [],
    "ak": "",
}));
const AUTH_HEADERS = {
    'Tardis-Context': TARDIS_CTX,
    'Content-Type': 'application/json',
};
const NON_BLOCKING_TOPIC = 'test/non-blocking';
const BLOCKING_TOPIC = 'test/blocking';

export {
    SERVER_URI,
    TARDIS_CTX,
    AUTH_HEADERS,
    NON_BLOCKING_TOPIC,
    BLOCKING_TOPIC
};

