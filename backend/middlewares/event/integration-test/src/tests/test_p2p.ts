import { MessageTargetKind, newMessage } from 'asteroid-mq-sdk';
import { getNode } from "../api";
import { NON_BLOCKING_TOPIC } from "../consts";

type P2pMessageType = {
    data: string
}
const randomBytes = new Uint8Array(12);
crypto.getRandomValues(randomBytes);
const channelCode = btoa(String.fromCharCode(...randomBytes));
const channelSubject = `event/test_p2p/${channelCode}`;
const createP2pMessage = (data: string) => newMessage<P2pMessageType>(
    {
        "data": data
    }, {
    topic: NON_BLOCKING_TOPIC,
    subjects: [channelSubject],
    targetKind: MessageTargetKind.Online
});

const TEST_P2P_MESSAGE_COUNT = 100;

export default async () => {
    // 创建节点A
    const nodeA = await getNode();
    // 创建节点B
    const nodeB = await getNode();

    const endpointA = await nodeA.createEndpoint(NON_BLOCKING_TOPIC, [channelSubject]);
    const endpointB = await nodeB.createEndpoint(NON_BLOCKING_TOPIC, [channelSubject]);

    await endpointA.node.sendMessage(createP2pMessage('hello from A'));
    await endpointB.node.sendMessage(createP2pMessage('hello from B'));

    let successCount = 0;
    const sendTasks = [];
    for (let i = 0; i < TEST_P2P_MESSAGE_COUNT; i++) {
        let task = endpointA.node.sendMessage(createP2pMessage(`message ${i}`)).then(() => {
            successCount++;
        });
        sendTasks.push(task);
    }
    await Promise.all(sendTasks);
    console.log(`success_rate: ${successCount/TEST_P2P_MESSAGE_COUNT}, total: ${TEST_P2P_MESSAGE_COUNT}`);

    // 关闭节点A
    await nodeA.close();
    // 关闭节点B
    await nodeB.close();
    // 等待节点B的消息处理任务结束
}