package me.caelunshun.quicproxy.jni;

import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class RustQuicClient {
    private final long ptr;
    private Lock lock = new ReentrantLock();

    RustQuicClient(long ptr) {
        this.ptr = ptr;
    }

    public int getPort() {
        lock.lock();
        int result = getPort(ptr);
        lock.unlock();
        return result;
    }

    public void enableEncryption(byte[] key) {
        lock.lock();
        enableEncryption(ptr, key);
        lock.unlock();
    }

    @Override
    protected void finalize() {
        lock.lock();
        drop(ptr);
        lock.unlock();
    }

    private static native int getPort(long ptr);
    private static native void enableEncryption(long ptr, byte[] key);
    private static native void drop(long ptr);
}
