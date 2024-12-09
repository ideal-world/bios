// Very long connection test

import { newMessage, MessageTargetKind, Node } from 'asteroid-mq-sdk';
import { getNode } from '../api';
import * as consts from '../consts';
import type { Endpoint } from 'asteroid-mq-sdk/dist/src/endpoint';

interface TestMessage {
}
const topic = consts.NON_BLOCKING_TOPIC;
type MessageType = TestMessage;
const dataMessage = () => newMessage<MessageType>(
    {},
    {
        topic,
        subjects: ["event/test_durable_message"],
        targetKind: MessageTargetKind.Online,
    }
);
const TEST_CONNECTION_LIFESPAN = 12 * 60 * 60 * 1000;
const SEND_INTERVAL_MS = 60 * 1000;
function timer(ms: number) {
    return new Promise<void>((resolve) => {
        setTimeout(() => {
            resolve();
        }, ms);
    });
}
export default async () => {
    const nodeA = await getNode();
    const nodeB = await getNode();
    const nodeC = await getNode();
    const epA = await nodeA.createEndpoint(topic, ["event/long_connection"]);
    const epB = await nodeB.createEndpoint(topic, ["event/long_connection"]);
    const epC = await nodeC.createEndpoint(topic, ["event/long_connection"]);
    let quit = false;
    const sendTask = async (node: Node) => {
        while (quit === false) {
            await node.sendMessage(dataMessage());
            await timer(SEND_INTERVAL_MS);
        }
    }
    const recvTask = async (ep: Endpoint) => {
        for await (const message of ep.messages()) {
            console.log(`Node A received message: ${message}`);
        }
    }
    Promise.all([
        sendTask(nodeA),
        sendTask(nodeB),
        sendTask(nodeC),
        recvTask(epA),
        recvTask(epB),
        recvTask(epC),
    ]);

    await timer(TEST_CONNECTION_LIFESPAN);
    quit = true;
};