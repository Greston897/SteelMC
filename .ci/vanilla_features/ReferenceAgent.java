import java.io.*;
import java.lang.instrument.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.security.ProtectionDomain;
import java.util.*;
import java.util.jar.JarFile;
import java.util.zip.GZIPOutputStream;
import jdk.internal.org.objectweb.asm.*;

/** Capture vanilla block data, never replace Steel's expected hash fixture. */
public class ReferenceAgent {
    public static void premain(String jar, Instrumentation instrumentation) throws Exception {
        // The bundled server uses a loader whose parent is the platform loader.
        // A bootstrap helper accepting Object is visible without linking game classes.
        JarFile archive = new JarFile(jar);
        // Keep the transformer and all its nested visitors in the agent loader.
        // The duplicate bootstrap class is used only by the public Object hook.
        for (var entry : Collections.list(archive.entries())) {
            String name = entry.getName();
            if (name.startsWith("ReferenceAgent$") && name.endsWith(".class")) {
                Class.forName(name.substring(0, name.length() - 6).replace('/', '.'), true,
                              ReferenceAgent.class.getClassLoader());
            }
        }
        instrumentation.appendToBootstrapClassLoaderSearch(archive);
        instrumentation.addTransformer(new ClassFileTransformer() {
            public byte[] transform(Module module, ClassLoader loader, String name, Class<?> type,
                                    ProtectionDomain domain, byte[] bytes) {
                if (!name.equals("net/minecraft/server/MinecraftServer")) return null;
                ClassReader reader = new ClassReader(bytes);
                ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_MAXS);
                reader.accept(new ClassVisitor(Opcodes.ASM8, writer) {
                    public MethodVisitor visitMethod(int access, String name, String desc,
                                                     String signature, String[] exceptions) {
                        MethodVisitor target = super.visitMethod(access, name, desc, signature, exceptions);
                        if (!name.equals("runServer") || !desc.equals("()V")) return target;
                        return new MethodVisitor(Opcodes.ASM8, target) {
                            public void visitCode() {
                                super.visitCode();
                                visitVarInsn(Opcodes.ALOAD, 0);
                                visitMethodInsn(Opcodes.INVOKESTATIC, "ReferenceAgent", "capture",
                                                "(Ljava/lang/Object;)V", false);
                            }
                        };
                    }
                }, 0);
                System.out.println("REFERENCE hooked server startup; generation code is unchanged");
                return writer.toByteArray();
            }
        });
    }

    public static void capture(Object server) {
        new Thread(() -> {
            try {
                Method ready = server.getClass().getMethod("isReady");
                long deadline = System.nanoTime() + 600_000_000_000L;
                while (!(Boolean) ready.invoke(server)) {
                    if (System.nanoTime() > deadline) throw new IOException("server startup deadline");
                    Thread.sleep(100);
                }
                server.getClass().getMethod("execute", Runnable.class).invoke(server, (Runnable) () -> {
                    try {
                        extract(server);
                        System.out.println("REFERENCE capture completed");
                        System.exit(0);
                    } catch (Throwable error) {
                        error.printStackTrace();
                        System.exit(2);
                    }
                });
            } catch (Throwable error) {
                error.printStackTrace();
                System.exit(2);
            }
        }, "vanilla-reference-capture").start();
    }

    private static List<int[]> positions(String name) throws IOException {
        List<int[]> result = new ArrayList<>();
        for (String line : Files.readAllLines(Path.of(name))) {
            String[] parts = line.split("\\s+");
            result.add(new int[]{Integer.parseInt(parts[0]), Integer.parseInt(parts[1])});
        }
        return result;
    }

    private static void extract(Object server) throws Exception {
        ClassLoader loader = server.getClass().getClassLoader();
        Class<?> levelClass = Class.forName("net.minecraft.world.level.Level", true, loader);
        Class<?> keyClass = Class.forName("net.minecraft.resources.ResourceKey", true, loader);
        Object level = server.getClass().getMethod("getLevel", keyClass)
                .invoke(server, levelClass.getField("OVERWORLD").get(null));
        Object cache = level.getClass().getMethod("getChunkSource").invoke(level);
        Class<?> statusClass = Class.forName("net.minecraft.world.level.chunk.status.ChunkStatus", true, loader);
        Object features = statusClass.getField("FEATURES").get(null);
        Method getChunk = cache.getClass().getMethod("getChunk", int.class, int.class, statusClass, boolean.class);
        List<int[]> tracked = positions("tracked.tsv");
        int count = 0;
        for (int[] pos : tracked) {
            Objects.requireNonNull(getChunk.invoke(cache, pos[0], pos[1], features, true));
            if (++count % 10 == 0) System.out.println("REFERENCE generated " + count + "/" + tracked.size());
        }
        List<int[]> dumps = positions("dump.tsv");
        Class<?> stateClass = Class.forName("net.minecraft.world.level.block.state.BlockState", true, loader);
        Method getId = Class.forName("net.minecraft.world.level.block.Block", true, loader)
                .getMethod("getId", stateClass);
        try (DataOutputStream out = new DataOutputStream(new GZIPOutputStream(Files.newOutputStream(Path.of("reference.bin.gz"))))) {
            out.writeInt(dumps.size());
            for (int[] pos : dumps) {
                Object chunk = getChunk.invoke(cache, pos[0], pos[1], features, true);
                Object[] sections = (Object[]) chunk.getClass().getMethod("getSections").invoke(chunk);
                out.writeInt(pos[0]); out.writeInt(pos[1]); out.writeInt(sections.length);
                MessageDigest hash = MessageDigest.getInstance("MD5");
                for (Object section : sections) {
                    boolean empty = (Boolean) section.getClass().getMethod("hasOnlyAir").invoke(section);
                    out.writeByte(empty ? 0 : 1);
                    if (empty) { hash.update((byte) 0); continue; }
                    Method getState = section.getClass().getMethod("getBlockState", int.class, int.class, int.class);
                    for (int y = 0; y < 16; y++) for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
                        int id = (Integer) getId.invoke(null, getState.invoke(section, x, y, z));
                        out.writeInt(id);
                        hash.update(new byte[]{(byte)(id >>> 24), (byte)(id >>> 16), (byte)(id >>> 8), (byte) id});
                    }
                }
                System.out.println("REFERENCE hash " + pos[0] + " " + pos[1] + " " + HexFormat.of().formatHex(hash.digest()));
            }
        }
    }
}
