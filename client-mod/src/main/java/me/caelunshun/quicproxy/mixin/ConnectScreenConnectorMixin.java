package me.caelunshun.quicproxy.mixin;

import io.netty.channel.ChannelFuture;
import me.caelunshun.quicproxy.ConnectScreenExt;
import me.caelunshun.quicproxy.ConnectionExt;
import me.caelunshun.quicproxy.ServerAddressExt;
import me.caelunshun.quicproxy.client.ConnectionType;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.ConnectScreen;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import net.minecraft.network.Connection;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.net.InetSocketAddress;

@Mixin(targets = "net.minecraft.client.gui.screens.ConnectScreen$1")
public class ConnectScreenConnectorMixin {
    private ConnectScreen parent;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void initWithParent(ConnectScreen connectScreen, String string, ServerAddress serverAddress,
                                Minecraft minecraft, ServerData serverData, CallbackInfo ci) {
        this.parent = connectScreen;
    }

    @Redirect(method = "run", at = @At(value = "INVOKE", target = "Lnet/minecraft/network/Connection;connect(Ljava/net/InetSocketAddress;ZLnet/minecraft/network/Connection;)Lio/netty/channel/ChannelFuture;"))
    private ChannelFuture connect(InetSocketAddress address, boolean nativeTransport, Connection connection) {
        ServerAddress serverAddress = ((ConnectScreenExt) (Object) parent).getServerAddress();
        if (((ServerAddressExt) (Object) serverAddress).getConnectionType() == ConnectionType.NORMAL) {
            return Connection.connect(address, nativeTransport, connection);
        } else {
            return ((ConnectionExt) connection).connectViaQuic(address);
        }
    }
}
