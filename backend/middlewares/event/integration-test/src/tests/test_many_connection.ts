import { MessageTargetKind, newMessage } from 'asteroid-mq-sdk';
import { getNode } from '../api';
import * as consts from '../consts';
import ProgressBar from 'progress';
import { sleep } from '../utils';

interface TestMessage {
}
const topic = consts.NON_BLOCKING_TOPIC;
type MessageType = TestMessage;
const dataMessage = () => newMessage<MessageType>(
    {},
    {
        topic,
        subjects: ["event/test_many_connection"],
        targetKind: MessageTargetKind.Push,
    }
);
const CONNECTION_COUNT = 1000;
const CONNECTION_INTERVAL_MS = 100;
const SEND_INTERVAL_MS = 500;
const MESSAGE_COUNT = 1000;

export default async () => {
    const senderNode = await getNode();
    const nodes = [];
    let sendProgress = new ProgressBar('send: [:bar] :percent :current/:total', {
        total: MESSAGE_COUNT,
        width: 50
    });
    let connectProgress = new ProgressBar('connect: [:bar] :percent :current/:total', {
        total: CONNECTION_COUNT,
        width: 50
    });
    for (let i = 0; i < CONNECTION_COUNT; i++) {
        const node = await getNode();
        nodes.push(node);
        const _ep = node.createEndpoint(topic, ["event/test_many_connection"]);
        connectProgress.update(i / CONNECTION_COUNT);
        await sleep(CONNECTION_INTERVAL_MS)
    }
    for (let i = 0; i < MESSAGE_COUNT; i++) {
        senderNode.sendMessage(dataMessage());
        sendProgress.update(i / MESSAGE_COUNT);
        await sleep(SEND_INTERVAL_MS);
    }
    for (const node of nodes) {
        await node.close();
    }
};