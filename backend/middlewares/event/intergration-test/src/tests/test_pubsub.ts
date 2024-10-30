import { MessageTargetKind, newMessage } from "asteroid-mq-sdk";
import { getNode } from "../api";
import { NON_BLOCKING_TOPIC } from "../consts";

type P2pMessageType = {
    data: string
} | 'quit';


const createP2pMessage = (data: string) => newMessage<P2pMessageType>(
    {
        "data": data
    }, {
    topic: NON_BLOCKING_TOPIC,
    subjects: ["event/test_pubsub"],
    targetKind: MessageTargetKind.Online
});

const createQuitMessage = () => newMessage<P2pMessageType>(
    'quit', {
    topic: NON_BLOCKING_TOPIC,
    subjects: ["event/test_pubsub"],
    targetKind: MessageTargetKind.Available
});

const TEST_PUSBUB_MESSAGE_COUNT = 100;
const TEST_SUBSCRIBER_COUNT = 100;

export default async () => {

    const pubNode = await getNode();
    let pubCnt = 0;
    let recvCnt = 0;
    const subNode = async (_: number) => {
        const node = await getNode();
        const ep = await node.createEndpoint(NON_BLOCKING_TOPIC, ["event/test_pubsub"]);
        let messageStream = ep.messages();
        const process = new Promise((resolve, reject) => {
            (
                async () => {
                    try {
                        for await (const message of messageStream) {
                            if (message !== undefined) {
                                message.received();
                                const payload = message.json<P2pMessageType>();
                                if (payload === 'quit') {
                                    resolve(undefined);
                                    break;
                                } else {
                                    recvCnt++;
                                }
                                message.processed();
                            } else {
                                resolve(undefined);
                                break;
                            }
                        }
                    } catch (error) {
                        reject(error);
                    }
                }
            )();
        });
        await process;
    }
    const subNodeTasks = [];
    for (let i = 0; i < TEST_SUBSCRIBER_COUNT; i++) {
        const task = subNode(i);
        subNodeTasks.push(task);
    }


    const sendTasks = [];
    for (let i = 0; i < TEST_PUSBUB_MESSAGE_COUNT; i++) {
        let task = pubNode.sendMessage(createP2pMessage('hello from pub')).then(() => {
            pubCnt++;
        });
        sendTasks.push(task);
    }
    await Promise.all(sendTasks);
    console.log(`success_rate: ${recvCnt / pubCnt}, total: ${TEST_PUSBUB_MESSAGE_COUNT}`);
    await pubNode.sendMessage(createQuitMessage());
    await pubNode.close();
    await Promise.all(subNodeTasks);
}