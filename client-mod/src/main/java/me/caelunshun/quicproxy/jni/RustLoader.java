package me.caelunshun.quicproxy.jni;

import org.apache.commons.io.IOUtils;

import java.io.*;

public class RustLoader {
    public static void loadNativeLibraries() throws IOException {
        String os = System.getProperty("os.name").toLowerCase();
        String resourceName;
        String fileExtension;
        if (os.contains("win")) {
            resourceName = "minecraft_quic_proxy_jni";
            fileExtension = "dll";
        } else if (os.contains("nix") || os.contains("nux") || os.contains("aix")) {
            resourceName = "libminecraft_quic_proxy_jni";
            fileExtension = "so";
        } else if (os.contains("mac")) {
            resourceName = "libminecraft_quic_proxy_jni";
            fileExtension = "dylib";
        } else {
            throw new RuntimeException("unsupported OS: " + os);
        }

        try (InputStream stream = RustLoader.class.getClassLoader().getResourceAsStream(resourceName + "." + fileExtension)) {
            File tempFile = File.createTempFile("quic-proxy", "jni-library." + fileExtension);
            tempFile.deleteOnExit();
            try (OutputStream outputStream = new FileOutputStream(tempFile)) {
                IOUtils.copy(stream, outputStream);
            }

            System.load(tempFile.getPath());
        }
    }
}
