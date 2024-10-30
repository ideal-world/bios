import { MessageTargetKind, Node, newMessage } from 'asteroid-mq-sdk';
import { fetchConnectUrl } from '../api';
import { NON_BLOCKING_TOPIC } from '../consts';

interface TestMessage {
    "data": string
}

const createMessage = (data: string) => newMessage<TestMessage>(
    {
        "data": data
    }, {
    topic: NON_BLOCKING_TOPIC,
    subjects: ["event/test_different_size"],
    targetKind: MessageTargetKind.Online
});

const generateData = (sizeInKB: number) => {
    return 'a'.repeat(sizeInKB * 1024);
};

export default async () => {
    const nodeA = Node.connect({
        url: await fetchConnectUrl()
    });
    const nodeB = Node.connect({
        url: await fetchConnectUrl()
    });

    const endpointB = await nodeB.createEndpoint(NON_BLOCKING_TOPIC, ["event/test"]);

    const processTaskB = new Promise((resolve, reject) => {
        (
            async () => {
                try {
                    for await (const message of endpointB.messages()) {
                        if (message !== undefined) {
                            const payload = message.json<TestMessage>();
                            message.received();
                            console.log(`Received message of size: ${payload.data.length / 1024} KB`);
                            message.processed();
                        } else {
                            resolve(undefined);
                        }
                    }
                } catch (error) {
                    reject(error);
                }
            }
        )();
    });

    // 发送小数据包（1KB）
    await nodeA.sendMessage(createMessage(generateData(1)));
    // 发送大数据包（10MB）
    await nodeA.sendMessage(createMessage(generateData(10240)));

    await nodeA.close();
    await nodeB.close();
    await processTaskB;

};