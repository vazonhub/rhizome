import { PeerId } from './peer_id';
import { UdpPeer } from './udp_peer';

// (async function() {
//     console.log("------ Starting UDP Peer-to-Peer Communication Example ------");

//     // Создаем два UDP пира на разных портах
//     const peer1 = new UdpPeer(4000);
//     const peer2 = new UdpPeer(4001);

//     // Отправляем сообщение от peer1 к peer2
//     peer1.sendMessage("Hello from Peer 1!", "127.0.0.1", 4001);

//     // Отправляем сообщение от peer2 к peer1
//     peer2.sendMessage("Hi from Peer 2!", "127.0.0.1", 4000);

//     // Через некоторое время закрываем пиры
//     setTimeout(() => {
//         peer1.close();
//         peer2.close();
//     }, 5000);
// })();

(async function() {
    console.log("\n\n------ Starting Peer ID Example ------");

    // Пример использования:
    const peerA = new PeerId();
    const peerB = new PeerId();

    console.log(`Peer A ID: ${peerA.id}, Public Key: ${peerA.publicKey.slice(0, 50)}...`);
    console.log(`Peer B ID: ${peerB.id}, Public Key: ${peerB.publicKey.slice(0, 50)}...`);

    const message = "Hello from Peer A!";
    const signature = peerA.sign(message);
    console.log(`Signature from A: ${signature.slice(0, 20)}...`);

    const isValid = peerB.verify(message, signature, peerA.publicKey);
    console.log(`Is signature valid by B? ${isValid}`); // Должно быть true

    const tamperedMessage = "Hello from Peer A! (tampered)";
    const isTamperedValid = peerB.verify(tamperedMessage, signature, peerA.publicKey);
    console.log(`Is tampered signature valid by B? ${isTamperedValid}`); // Должно быть false
})();