import { Node, newMessage, MessageTargetKind } from 'asteroid-mq-sdk';
import { fetchConnectUrl } from '../api';
import { NON_BLOCKING_TOPIC } from '../consts';
import { sleep } from 'bun';

interface TestMessage {
    data: string;
}

type MessageType = TestMessage | 'quit';
const dataMessage = (data: string) => newMessage<MessageType>(
    { data },
    {
        topic: NON_BLOCKING_TOPIC,
        subjects: ["event/test_many_nodes"],
        targetKind: MessageTargetKind.Push
    }
);

const quitMessage = newMessage<MessageType>(
    'quit',
    {
        topic: NON_BLOCKING_TOPIC,
        subjects: ["event/test_many_nodes"],
        targetKind: MessageTargetKind.Online
    }
);
const SENDER_CONCURRENT_SIZE = 100;
const RECEIVER_CONCURRENT_SIZE = 100;
const TOTAL_MESSAGE_COUNT = 1000;

export default async () => {
    const hostNode = Node.connect({
        url: await fetchConnectUrl()
    });
    const recvNode = async (_nodeIdx: number) => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });
        const ep = await node.createEndpoint(NON_BLOCKING_TOPIC, ["event/test_many_nodes"]);
        for await (const message of ep.messages()) {
            if (message !== undefined) {
                message.received();
                const payload = message.json<MessageType>();
                message.processed();
                if (payload === 'quit') {
                    break;
                }
            } else {
                break;
            }
        }
    }
    const senderNode = async (nodeIdx: number) => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });
        let sendCount = 0;
        let errorCount = 0;
        for (let idx = 0; idx < TOTAL_MESSAGE_COUNT; idx++) {
            try {
                await node.sendMessage(dataMessage(`message-${nodeIdx}-${idx}`));
            } catch (_) {
                errorCount++;
            } finally {
                sendCount++;
            }
        }
        return errorCount / sendCount
    }
    // 创建接收节点
    let recvTaskSet = [];
    let sendTaskSet = [];
    for (let idx = 0; idx < SENDER_CONCURRENT_SIZE; idx++) {
        recvTaskSet.push(recvNode(idx))
    }
    await sleep(1000);
    console.time('dispatch-messages');
    for (let idx = 0; idx < RECEIVER_CONCURRENT_SIZE; idx++) {
        sendTaskSet.push(senderNode(idx))
    }
    const fault_rate = await Promise.allSettled(sendTaskSet);
    console.timeEnd('dispatch-messages');
    const fulfilled_part = fault_rate.filter((result) => result.status === 'fulfilled').map((result) => result.value);
    const average_fault_rate = fulfilled_part.reduce((acc, cur) => acc + cur, 0) / fulfilled_part.length;
    const rejected_part = fault_rate.filter((result) => result.status === 'rejected').map((result) => result.reason);
    const reject_rate = rejected_part.length / SENDER_CONCURRENT_SIZE;
    console.log(`average fault rate: ${average_fault_rate}`);
    console.log(`rejected rate: ${reject_rate}`);
    await hostNode.sendMessage(quitMessage);
    await Promise.allSettled(recvTaskSet);
};