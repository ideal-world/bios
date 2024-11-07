import { Node, newMessage, MessageTargetKind, MessageAckExpectKind } from 'asteroid-mq-sdk';
import { fetchConnectUrl } from '../api';
import * as consts from '../consts';

interface TestMessage {
    index: number;
}
const topic = consts.BLOCKING_TOPIC;
const ORDER_TEST_MESSAGE_COUNT = 50;
type MessageType = TestMessage | 'quit';
const dataMessage = (index: number) => newMessage<MessageType>(
    { index },
    {
        topic,
        subjects: ["event/test_order"],
        targetKind: MessageTargetKind.Push,
        ackKind: MessageAckExpectKind.Processed
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
    const hostNode = Node.connect({
        url: await fetchConnectUrl()
    });

    const senderNode = async () => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });

        for (let idx = 0; idx < ORDER_TEST_MESSAGE_COUNT; idx++) {
            try {
                await node.sendMessage(dataMessage(idx));
            } catch (_) {
                console.warn(`send message failed: ${idx}`);
            } finally {
            }
        }
    }
    // 创建接收节点
    const recvNode = async () => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });
        const ep = await node.createEndpoint(topic, ["event/test_order"]);
        const recvTask = new Promise((resolve, reject) => {
            (
                async () => {
                    try {
                        let lastIdx = -1;
                        for await (const message of ep.messages()) {
                            if (message !== undefined) {
                                await message.received();
                                const payload = message.json<MessageType>();
                                if (payload === 'quit') {
                                    resolve(undefined);
                                    return;
                                }
                                if (payload.index !== lastIdx + 1) {
                                    console.warn(`out of order: ${lastIdx} -> ${payload.index}`);
                                }
                                lastIdx = payload.index;
                                await message.processed();
                            } else {
                                resolve(undefined);
                                return;
                            }
                        }
                    } catch (error) {
                        reject(error);
                    }
                }
            )();
        });
        return recvTask;
    }
    let taskSend = senderNode();
    let taskRecv = recvNode();
    await taskSend;
    await hostNode.sendMessage(quitMessage);
    await taskRecv;
};