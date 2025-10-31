import { generateKeyPairSync, sign as cryptoSign, verify as cryptoVerify } from 'crypto';

export class PeerId {
    readonly publicKey: string;
    readonly privateKey: string;
    readonly id: string;

    constructor() {
        // Генерируем Ed25519 пару ключей
        const { publicKey, privateKey } = generateKeyPairSync('ed25519', {
            publicKeyEncoding: { type: 'spki', format: 'pem' },
            privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
        });

        this.publicKey = publicKey.toString();
        this.privateKey = privateKey.toString();
        this.id = this.publicKey.slice(27, 40) + '...';
    }

    sign(message: string | Buffer): string {
        const signature = cryptoSign(null, Buffer.from(message), this.privateKey);
        return signature.toString('base64');
    }

    verify(message: string | Buffer, signature: string, publicKey: string): boolean {
        return cryptoVerify(null, Buffer.from(message), publicKey, Buffer.from(signature, 'base64'));
    }
}