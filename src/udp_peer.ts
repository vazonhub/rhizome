// src/udp_peer.ts
import * as dgram from 'dgram';

export class UdpPeer {
    private socket: dgram.Socket;
    private port: number;

    constructor(port: number) {
        this.port = port;
        this.socket = dgram.createSocket('udp4');

        this.socket.on('error', (err) => {
            console.error(`UDP Server error:\n${err.stack}`);
            this.socket.close();
        });

        this.socket.on('message', (msg, rinfo) => {
            console.log(`UDP Server received ${msg.toString()} from ${rinfo.address}:${rinfo.port}`);
            // Здесь будет логика обработки сообщения
            this.handleMessage(msg, rinfo);
        });

        this.socket.on('listening', () => {
            const address = this.socket.address();
            console.log(`UDP Server listening ${address.address}:${address.port}`);
        });

        this.socket.bind(this.port);
    }

    private handleMessage(msg: Buffer, rinfo: dgram.RemoteInfo): void {
        // В будущем здесь будет парсинг пакетов DHT или других сообщений протокола
        console.log(`  Processed message: ${msg.toString()}`);
    }

    // Метод для отправки UDP-сообщений
    sendMessage(message: string | Buffer, remoteAddress: string, remotePort: number): void {
        const buffer = typeof message === 'string' ? Buffer.from(message) : message;
        this.socket.send(buffer, remotePort, remoteAddress, (err) => {
            if (err) {
                console.error(`Error sending UDP message: ${err.stack}`);
            } else {
                console.log(`Sent "${buffer.toString()}" to ${remoteAddress}:${remotePort}`);
            }
        });
    }

    close(): void {
        this.socket.close();
        console.log(`UDP Server on port ${this.port} closed.`);
    }
}