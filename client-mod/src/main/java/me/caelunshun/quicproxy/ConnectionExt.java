package me.caelunshun.quicproxy;

import io.netty.channel.ChannelFuture;
import me.caelunshun.quicproxy.client.ConnectionType;

import java.net.InetSocketAddress;

public interface ConnectionExt {
    ChannelFuture connectViaQuic(InetSocketAddress destinationServer);
}
