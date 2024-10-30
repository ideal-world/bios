import { newMessage, MessageTargetKind } from 'asteroid-mq-sdk';
import { getNode } from '../api';
import * as consts from '../consts';
import type { MessageConfig } from 'asteroid-mq-sdk/dist/message';

interface TestMessage {
}
const topic = consts.BLOCKING_TOPIC;
type MessageType = TestMessage;
const dataMessage = (config: Exclude<MessageConfig['durability'], undefined>) => newMessage<MessageType>(
    {},
    {
        topic,
        subjects: ["event/test_durable_message"],
        targetKind: MessageTargetKind.Durable,
        durability: config
    }
);


export default async () => {
    const hostNode = await getNode();
    const receiverA = await getNode();
    const receiverB = await getNode();
    const receiverC = await getNode();
    let sendTask = hostNode.sendMessage(dataMessage({
        expire: new Date(Date.now() + 60 * 1000),
        maxReceiver: 3
    }));
    const epA = await receiverA.createEndpoint(topic, ["event/test_durable_message"]);
    const epAMessages = epA.messages();
    await epAMessages.next();
    const epB = await receiverB.createEndpoint(topic, ["event/test_durable_message"]);
    const epBMessages = epB.messages();
    await epBMessages.next();
    const epC = await receiverC.createEndpoint(topic, ["event/test_durable_message"]);
    const epCMessages = epC.messages();
    await epCMessages.next();
    await sendTask;
    await hostNode.close();
    await receiverA.close();
    await receiverB.close();
    await receiverC.close();
};