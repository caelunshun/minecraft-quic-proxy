package me.caelunshun.quicproxy.jni;

public class RustQuicContext {
    private final long ptr;

    public RustQuicContext() {
        ptr = init();
    }

    public RustQuicClient createClient(String gatewayHost, int gatewayPort,
                                       String destinationServerAddress, String authenticationKey) {
        return new RustQuicClient(createClient(ptr, gatewayHost, gatewayPort, destinationServerAddress, authenticationKey));
    }

    @Override
    protected void finalize() {
        drop(ptr);
    }

    private static native long init();
    private static native long createClient(long ptr, String gatewayHost, int gatewayPort,
                                            String destinationServerAddress, String authenticationKey);
    private static native void drop(long ptr);
}
