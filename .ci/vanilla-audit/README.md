# Vanilla 26.3 source audit baseline

Source mirror: https://github.com/0khs/minecraft_source_26.3/tree/74720c2890f22942b72d7d84958eff1b3f72b95b

`source-files-26.3.tsv` lists **every net/minecraft Java source file** from the complete (not truncated) Git tree at the pinned revision, including client files. Blob SHA identifies exact input. All entries initially remain NOT_AUDITED: enumeration is not an audit, and a Java class is not one mechanic. Client-only classification must be reviewed rather than inferred as a reason to exclude server mechanics.

Per-mechanic evidence must record vanilla class/method, rules and edge cases (including RNG, collision, fluid, tick timing, persistence, synchronization), Steel runtime path, discrepancy, fix, executed comparison/test, and remaining gaps. Static registration or LLVM execution is not parity. Do not turn a whole source file PASS after testing one method. Steel files outside LLVM remain in scope even if no directly corresponding vanilla class exists.

Current partial comparisons: `SaplingBlock`, `AzaleaBlock`, `TreeGrower`; findings and unverified fixes are recorded in `HEADLESS-VERIFICATION-STATUS-RU.md`. Their source-file rows remain NOT_AUDITED because these files have not passed a complete method/branch/runtime audit.
