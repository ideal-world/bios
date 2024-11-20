import { Node, newMessage, MessageTargetKind, } from 'asteroid-mq-sdk';
import { fetchConnectUrl } from '../api';
import { NON_BLOCKING_TOPIC } from '../consts';
import { sleep } from 'bun';
import ProgressBar from 'progress';
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
const TOTAL_MESSAGE_COUNT = 10000;
const EXPECT_TOTAL_COUNT = SENDER_CONCURRENT_SIZE * TOTAL_MESSAGE_COUNT;
const PROGRESS_UNIT = EXPECT_TOTAL_COUNT / 100;
export default async () => {
    const hostNode = Node.connect({
        url: await fetchConnectUrl()
    });
    let totalSendCount = 0;
    let totalRecvCount = 0;
    let sendProgress = new ProgressBar('send: [:bar] :percent :current/:total', {
        total: EXPECT_TOTAL_COUNT,
        width: 50
    });
    let recvProgress = new ProgressBar('recv: [:bar] :percent :current/:total', {
        total: EXPECT_TOTAL_COUNT,
        width: 50
    });
    const recvNode = async (_nodeIdx: number) => {
        const node = Node.connect({
            url: await fetchConnectUrl()
        });
        const ep = await node.createEndpoint(NON_BLOCKING_TOPIC, ["event/test_many_nodes"]);
        for await (const message of ep.messages()) {
            if (message !== undefined) {
                totalRecvCount += 1;
                recvProgress.update(totalRecvCount / EXPECT_TOTAL_COUNT);
                if (totalRecvCount % PROGRESS_UNIT === 0) {
                    recvProgress.tick();
                }
                const payload = message.json<MessageType>();
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
                node.sendMessage(dataMessage(`message-${nodeIdx}-${idx}`))
                    .then(() => {
                        totalSendCount += 1;
                        sendProgress.update(totalSendCount / EXPECT_TOTAL_COUNT);
                        if (totalRecvCount % PROGRESS_UNIT === 0) {
                            sendProgress.tick();
                        }
                    });
            } catch (e) {
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
    console.log('all messages dispatched');

    console.timeEnd('dispatch-messages');
    const fulfilled_part = fault_rate.filter((result) => result.status === 'fulfilled').map((result) => result.value);
    const average_fault_rate = fulfilled_part.reduce((acc, cur) => acc + cur, 0) / fulfilled_part.length;
    const rejected_part = fault_rate.filter((result) => result.status === 'rejected').map((result) => result.reason);
    const reject_rate = rejected_part.length / SENDER_CONCURRENT_SIZE;
    console.log(`average fault rate: ${average_fault_rate}`);
    console.log(`rejected rate: ${reject_rate}`);
    hostNode.sendMessage(quitMessage);
    await Promise.allSettled(recvTaskSet);
};