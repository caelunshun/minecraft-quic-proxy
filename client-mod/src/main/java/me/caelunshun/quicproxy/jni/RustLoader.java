package me.caelunshun.quicproxy.jni;

import org.apache.commons.io.IOUtils;

import java.io.*;

public class RustLoader {
    public static void loadNativeLibraries() throws IOException {
        try (InputStream stream = RustLoader.class.getClassLoader().getResourceAsStream("minecraft_quic_proxy_jni.dll")) {
            File tempFile = File.createTempFile("quic-proxy", "jni-library.dll");
            tempFile.deleteOnExit();
            try (OutputStream outputStream = new FileOutputStream(tempFile)) {
                IOUtils.copy(stream, outputStream);
            }

            System.load(tempFile.getPath());
        }
    }
}
