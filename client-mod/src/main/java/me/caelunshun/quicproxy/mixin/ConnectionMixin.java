package me.caelunshun.quicproxy.mixin;

import io.netty.channel.ChannelFuture;
import me.caelunshun.quicproxy.ConnectionExt;
import me.caelunshun.quicproxy.client.ConnectionType;
import me.caelunshun.quicproxy.client.QUICProxyClient;
import me.caelunshun.quicproxy.jni.RustQuicClient;
import net.minecraft.network.Connection;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import javax.crypto.Cipher;
import java.net.InetSocketAddress;

@Mixin(Connection.class)
public class ConnectionMixin implements ConnectionExt {
    private ConnectionType type = ConnectionType.NORMAL;
    private RustQuicClient quicClient = null;

    @Override
    public ChannelFuture connectViaQuic(InetSocketAddress destinationServer) {
        this.type = ConnectionType.QUIC;
        String address = destinationServer.getAddress().getHostAddress() + ":" + destinationServer.getPort();
        this.quicClient = QUICProxyClient.instance.getQuicContext()
                .createClient(address, "temp");

        InetSocketAddress clientAddr = new InetSocketAddress("127.0.0.1", quicClient.getPort());

        Connection conn = (Connection) (Object) this;
        return Connection.connect(clientAddr, true, conn);
    }

    @Inject(method = "setEncryptionKey", at = @At("HEAD"), cancellable = true)
    public void dontSetEncryptionKeyIfQuic(Cipher cipher1, Cipher cipher2, CallbackInfo callbackInfo) {
        if (type == ConnectionType.QUIC) {
            // Hack: in the MC protocol the key is the same as the IV, so we can get it from the Cipher :)
            quicClient.enableEncryption(cipher1.getIV());
            // Don't enable encryption on our side (only the gateway<=>destination connection is encrypted
            // with MC encryption)
            callbackInfo.cancel();
        }
    }

    @Inject(method = "setupCompression", at = @At("HEAD"), cancellable = true)
    public void dontEnableCompressionIfQuic(CallbackInfo callbackInfo) {
        if (type == ConnectionType.QUIC) {
            // No point in compressing the connection to the local QUIC client
            callbackInfo.cancel();
        }
    }
}
