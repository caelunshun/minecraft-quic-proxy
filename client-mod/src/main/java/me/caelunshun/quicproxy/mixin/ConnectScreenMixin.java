package me.caelunshun.quicproxy.mixin;

import me.caelunshun.quicproxy.ConnectScreenExt;
import me.caelunshun.quicproxy.ServerAddressExt;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.ConnectScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ConnectScreen.class)
public class ConnectScreenMixin implements ConnectScreenExt {
    private ServerAddress serverAddress;

    @Override
    public ServerAddress getServerAddress() {
        return serverAddress;
    }

    @Inject(method = "connect", at = @At("HEAD"))
    private void connectIntercept(final Minecraft minecraft, final ServerAddress serverAddress,
                                  final @Nullable ServerData serverData, CallbackInfo callbackInfo) {
        this.serverAddress = serverAddress;
    }
}
