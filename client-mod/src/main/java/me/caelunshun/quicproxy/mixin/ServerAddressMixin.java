package me.caelunshun.quicproxy.mixin;

import com.google.common.net.HostAndPort;
import me.caelunshun.quicproxy.ServerAddressExt;
import me.caelunshun.quicproxy.client.ConnectionType;
import me.caelunshun.quicproxy.client.QUICProxyClient;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import org.spongepowered.asm.mixin.*;

import java.net.IDN;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Mixin(ServerAddress.class)
public abstract class ServerAddressMixin implements ServerAddressExt {
    private ConnectionType connectionType;
    // The following fields are only set if connectionType == ConnectionType.QUIC.
    private String gatewayAddress;
    private int gatewayPort;
    private String authenticationKey;


    @Override
    public ConnectionType getConnectionType() {
        return connectionType;
    }

    @Override
    public String getGatewayAddress() {
        return gatewayAddress;
    }

    @Override
    public int getGatewayPort() {
        return gatewayPort;
    }

    @Override
    public String getAuthenticationKey() {
        return authenticationKey;
    }

    @Final
    @Shadow
    private static ServerAddress INVALID;

    @Unique
    private static final Pattern QUIC_ADDR_PATTERN = Pattern.compile("^quic://(.+)@(.+)/([0-9]+)/(.+)$");

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
            String gatewayAddress = null;
            int gatewayPort = -1;
            String authenticationKey = null;

            Matcher matcher = QUIC_ADDR_PATTERN.matcher(string);
            if (matcher.find()) {
                connectionType = ConnectionType.QUIC;
                string = matcher.group(1);
                gatewayAddress = matcher.group(2);
                try {
                    gatewayPort = Integer.parseInt(matcher.group(3));
                } catch (NumberFormatException e) {
                    return INVALID;
                }
                authenticationKey = matcher.group(4);
            }

            try {
                HostAndPort hostAndPort = HostAndPort.fromString(string).withDefaultPort(25565);
                if (hostAndPort.getHost().isEmpty()) {
                    return INVALID;
                } else {
                    ServerAddress address = new ServerAddress(hostAndPort.getHost(), hostAndPort.getPort());
                    ServerAddressMixin mixinAddress = (ServerAddressMixin) (Object) address;
                    mixinAddress.connectionType = connectionType;
                    mixinAddress.gatewayAddress = gatewayAddress;
                    mixinAddress.gatewayPort = gatewayPort;
                    mixinAddress.authenticationKey = authenticationKey;
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
