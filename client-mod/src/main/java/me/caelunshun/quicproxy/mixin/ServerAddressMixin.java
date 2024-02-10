package me.caelunshun.quicproxy.mixin;

import com.google.common.net.HostAndPort;
import me.caelunshun.quicproxy.ServerAddressExt;
import me.caelunshun.quicproxy.client.ConnectionType;
import me.caelunshun.quicproxy.client.QUICProxyClient;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.net.IDN;

@Mixin(ServerAddress.class)
public abstract class ServerAddressMixin implements ServerAddressExt {
    private ConnectionType connectionType;

    @Override
    public ConnectionType getConnectionType() {
        return connectionType;
    }

    @Final
    @Shadow
    private static ServerAddress INVALID;

    /**
     * @author caelunshun
     * @reason support QUIC prefix
     */
    @Overwrite
    public static ServerAddress parseString(String string) {
        if (string == null) {
            return INVALID;
        } else {
            ConnectionType connectionType = ConnectionType.NORMAL;
            if (string.startsWith(QUICProxyClient.ADDRESS_PREFIX)) {
                connectionType = ConnectionType.QUIC;
                string = string.replaceFirst(QUICProxyClient.ADDRESS_PREFIX, "");
            }

            try {
                HostAndPort hostAndPort = HostAndPort.fromString(string).withDefaultPort(25565);
                if (hostAndPort.getHost().isEmpty()) {
                    return INVALID;
                } else {
                    ServerAddress address = new ServerAddress(hostAndPort.getHost(), hostAndPort.getPort());
                    ((ServerAddressMixin) (Object) address).connectionType = connectionType;
                    System.out.println(connectionType.toString());
                    return address;
                }
            } catch (IllegalArgumentException var2) {
                return INVALID;
            }
        }
    }

    /**
     * @author caelunshun
     * @reason support QUIC prefix
     */
    @Overwrite
    public static boolean isValidAddress(String string) {
        if (string.startsWith(QUICProxyClient.ADDRESS_PREFIX)) {
            string = string.replaceFirst(QUICProxyClient.ADDRESS_PREFIX, "");
        }

        try {
            HostAndPort hostAndPort = HostAndPort.fromString(string);
            String string2 = hostAndPort.getHost();
            if (!string2.isEmpty()) {
                IDN.toASCII(string2);
                return true;
            }
        } catch (IllegalArgumentException ignored) {
        }

        return false;
    }
}
