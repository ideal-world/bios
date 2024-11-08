import { Node, newMessage, MessageTargetKind, MessageAckExpectKind } from 'asteroid-mq-sdk';
import { fetchConnectUrl } from '../api';
import * as consts from '../consts';

interface TestMessage {
    index: number;
}
const topic = consts.BLOCKING_TOPIC;
const ORDER_TEST_MESSAGE_COUNT = 20;
type MessageType = TestMessage | 'quit';
const dataMessage = (index: number) => newMessage<MessageType>(
    { index },
    {
        topic,
        subjects: ["event/test_order"],
        targetKind: MessageTargetKind.Push,
        ackKind: MessageAckExpectKind.Received
    }
);

const quitMessage = newMessage<MessageType>(
    'quit',
    {
        topic,
        subjects: ["event/test_order"],
        targetKind: MessageTargetKind.Online
    }
);

export default async () => {


    const senderNode = async () => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });

        for (let idx = 0; idx < ORDER_TEST_MESSAGE_COUNT; idx++) {
            try {
                await node.sendMessage(dataMessage(idx));
                console.log(`sent: ${idx}`);
            } catch (_) {
                console.warn(`send message failed: ${idx}`);
            } finally {
            }
        }
        node.sendMessage(quitMessage);
    }
    // 创建接收节点
    const node = Node.connect({
        url: await fetchConnectUrl()
    });
    const ep = await node.createEndpoint(topic, ["event/test_order"]);
    const recvNode = async () => {

        let lastIdx = -1;
        for await (const message of ep.messages()) {
            if (message !== undefined) {
                const payload = message.json<MessageType>();
                message.received();
                if (payload === 'quit') {
                    console.log('quit');
                    return;
                }
                console.log(`received: ${payload.index}`);
                if (payload.index !== lastIdx + 1) {
                    console.warn(`out of order: ${lastIdx} -> ${payload.index}`);
                }
                lastIdx = payload.index;
            } else {
                return;
            }
        }
        node.sendMessage(quitMessage);
    }


    let taskSend = senderNode();
    let taskRecv = recvNode();
    await taskSend;
    console.log('send done');
    await taskRecv;
    console.log('recv done');
};