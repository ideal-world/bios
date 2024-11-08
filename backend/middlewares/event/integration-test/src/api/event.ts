import { Node } from "asteroid-mq-sdk";
import { AUTH_HEADERS, SERVER_URI } from "../consts";

type TardisResponse = {
    code: string,
    msg: string,
    data: any
}
export const CONNECT_API = SERVER_URI + '/ca/connect';
export const fetchConnectUrl = async (): Promise<string> => {
    const nodeId = await registerNodeId();
    return CONNECT_API + '?node_id=' + nodeId;
}
export async function registerNodeId(): Promise<string> {
    const api = SERVER_URI + '/ca/register';
    const response = await fetch(api, {
        method: 'PUT',
        headers: AUTH_HEADERS,
    });
    const responseBody: TardisResponse = await response.json();
    const nodeId = responseBody.data.node_id;
    if (nodeId === undefined || nodeId === null) {
        throw new Error('Failed to register node id');
    }
    return nodeId;
}
export async function createTopic(code: string, blocking: boolean = false): Promise<void> {
    const api = SERVER_URI + '/ci/topic';
    const response = await fetch(api, {
        method: 'POST',
        headers: AUTH_HEADERS,
        body: JSON.stringify({
            "topic_code": code,
            "overflow_policy": "RejectNew",
            "overflow_size": 500,
            "check_auth": false,
            "blocking": blocking
        })
    });
    const responseBody: TardisResponse = await response.json();
    if (!responseBody.code.startsWith('2')) {
        console.error(responseBody);
        throw new Error(responseBody.msg);
    }
}
export async function getNode(): Promise<Node> {
    return Node.connect({
        url: await fetchConnectUrl()
    })
}