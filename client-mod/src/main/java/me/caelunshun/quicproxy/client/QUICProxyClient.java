package me.caelunshun.quicproxy.client;

import me.caelunshun.quicproxy.jni.RustLoader;
import me.caelunshun.quicproxy.jni.RustQuicContext;
import net.fabricmc.api.ClientModInitializer;

import java.io.IOException;

public class QUICProxyClient implements ClientModInitializer {
    public static final String ADDRESS_PREFIX = "quic://";

    private RustQuicContext quicContext;

    public static QUICProxyClient instance;

    @Override
    public void onInitializeClient() {
        instance = this;
        try {
            RustLoader.loadNativeLibraries();
        } catch (IOException e) {
            e.printStackTrace();
        }
        quicContext = new RustQuicContext();
    }

    public RustQuicContext getQuicContext() {
        return quicContext;
    }
}
