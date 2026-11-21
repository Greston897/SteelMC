import java.io.*;
import java.lang.instrument.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.security.ProtectionDomain;
import java.util.*;
import java.util.jar.JarFile;
import java.util.zip.GZIPOutputStream;
import java.lang.classfile.*;
import java.lang.constant.*;
import static java.lang.constant.ConstantDescs.*;

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
                if (name.equals("net/minecraft/world/level/levelgen/feature/SimpleBlockFeature")
                    && Boolean.getBoolean("steel.featuretrace")) {
                    ClassFile classFile = ClassFile.of();
                    return classFile.transformClass(classFile.parse(bytes),
                        ClassTransform.transformingMethodBodies(
                            method -> method.methodName().equalsString("place")
                                && method.methodType().equalsString("(Lnet/minecraft/world/level/WorldGenLevel;Lnet/minecraft/world/level/chunk/ChunkGenerator;Lnet/minecraft/util/RandomSource;Lnet/minecraft/core/BlockPos;)Z"),
                            new CodeTransform() {
                                @Override public void atStart(CodeBuilder code) {
                                    code.aload(0).aload(1).aload(4).invokestatic(ClassDesc.of("ReferenceAgent"), "traceFeature",
                                        MethodTypeDesc.of(CD_void, CD_Object, CD_Object, CD_Object));
                                }
                                @Override public void accept(CodeBuilder code, CodeElement element) { code.with(element); }
                            }));
                }
                if (!name.equals("net/minecraft/server/MinecraftServer")) return null;
                ClassFile classFile = ClassFile.of();
                byte[] transformed = classFile.transformClass(classFile.parse(bytes),
                    ClassTransform.transformingMethodBodies(
                        method -> method.methodName().equalsString("runServer")
                               && method.methodType().equalsString("()V"),
                        new CodeTransform() {
                            @Override public void atStart(CodeBuilder code) {
                                code.aload(0).invokestatic(ClassDesc.of("ReferenceAgent"), "capture",
                                    MethodTypeDesc.of(CD_void, CD_Object));
                            }
                            @Override public void accept(CodeBuilder code, CodeElement element) {
                                code.with(element);
                            }
                        }));
                System.out.println("REFERENCE hooked server startup; generation code is unchanged");
                return transformed;
            }
        });
    }

    private static final java.util.concurrent.atomic.AtomicBoolean featureHookConfirmed =
        new java.util.concurrent.atomic.AtomicBoolean();
    private static volatile Method featurePosX, featurePosY, featurePosZ;

    /** Optional CI diagnostic; never changes random state or generated blocks. */
    public static void traceFeature(Object feature, Object level, Object pos) {
        try {
            if (featureHookConfirmed.compareAndSet(false, true)) {
                System.out.println("FEATURE_TRACE_HOOK_OK");
            }
            if (featurePosX == null) {
                Class<?> blockPos = Class.forName("net.minecraft.core.BlockPos", true, pos.getClass().getClassLoader());
                featurePosY = blockPos.getMethod("getY");
                featurePosZ = blockPos.getMethod("getZ");
                featurePosX = blockPos.getMethod("getX");
            }
            int x = (Integer) featurePosX.invoke(pos);
            int y = (Integer) featurePosY.invoke(pos);
            int z = (Integer) featurePosZ.invoke(pos);
            if (x >= -6536443 && x <= -6536430 && y >= 72 && y <= 75
                && z >= -7689105 && z <= -7689090) {
                String details = "";
                if (feature.toString().contains("minecraft:flower_plain")) {
                    Object holder = feature.getClass().getMethod("toPlace").invoke(feature);
                    Class<?> holderClass = Class.forName("net.minecraft.core.Holder", true, pos.getClass().getClassLoader());
                    Object provider = holderClass.getMethod("value").invoke(holder);
                    Class<?> base = provider.getClass().getSuperclass();
                    Field scale = base.getDeclaredField("scale");
                    Field threshold = provider.getClass().getDeclaredField("threshold");
                    Class<?> blockPos = Class.forName("net.minecraft.core.BlockPos", true, pos.getClass().getClassLoader());
                    Method noiseValue = base.getDeclaredMethod("getNoiseValue", blockPos, double.class);
                    scale.setAccessible(true);
                    threshold.setAccessible(true);
                    noiseValue.setAccessible(true);
                    details = " noise=" + noiseValue.invoke(provider, pos, (double) scale.getFloat(provider))
                        + " threshold=" + threshold.getFloat(provider);
                }
                System.out.println("FEATURE_TRACE " + x + " " + y + " " + z + details + " " + feature);
            }
        } catch (ReflectiveOperationException error) {
            System.err.println("FEATURE_TRACE_ERROR " + error);
        }
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
                        // This callback runs on the server thread. System.exit waits
                        // for a shutdown hook that joins that same thread.
                        System.out.flush();
                        Runtime.getRuntime().halt(0);
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
