package me.caelunshun.quicproxy;

import me.caelunshun.quicproxy.client.ConnectionType;

public interface ServerAddressExt {
    ConnectionType getConnectionType();
    String getGatewayAddress();
    int getGatewayPort();
    String getAuthenticationKey();
}
