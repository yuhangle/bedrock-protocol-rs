# bedrock-protocol-rs

**Bedrock Protocol Toolchain** — Rust 工具链，用于 **Minecraft Bedrock Edition** 网络协议的序列化、反序列化与代码生成。
使用 Rust 实现，并导出 C FFI 供跨语言调用（Python、C++ 等）。

---

## 架构

```
                    ┌──────────────────────────┐
                    │  docs/protocol-docs/     │
                    │  (JSON: 190 packets,     │
                    │   108 enums, 196 types)  │
                    └────────┬─────────────────┘
                             │ cargo run --bin generate-data
                             ↓
                    ┌──────────────────────────┐
                    │  bedrock-protocol-data    │
                    │  data/v897.json (嵌入)     │
                    │  data/v924.json (嵌入)     │
                    │  data/v944.json (嵌入)     │
                    │  data/v975.json (嵌入)     │
                    │  data/v1001.json (嵌入)    │
                    └────────┬─────────────────┘
                             │ include_str!() at compile time
                             ↓
┌──────────────────────────────────────────────────────────────┐
│  bedrock-protocol-schema                                     │
│  Schema / FieldType / ProtocolRegistry / EmbeddedVersion     │
│  协议定义的运行时查询接口                                       │
└──────────────────────────────────────────────────────────────┘
         │                           │
         ↓                           ↓
┌──────────────────┐    ┌───────────────────────────┐
│  bedrock-codegen  │    │  bedrock-protocol          │
│  (build.rs 调用)   │    │  Packet trait, 手写类型,    │
│  enums/types      │    │  NBT, FFI                 │
│  /packets/factory │    │                           │
└──────────────────┘    └────────┬──────────────────┘
                                 │
                    ┌────────────┼────────────┐
                    ↓            ↓            ↓
           ┌────────────┐ ┌───────────┐ ┌──────────┐
           │ bedrock-   │ │ bedrock-  │ │ bedrock- │
           │ common     │ │ binary-   │ │ nbt      │
           │ (traits,   │ │ stream    │ │ (NBT     │
           │  errors,   │ │ (read/    │ │  encode/ │
           │  varint)   │ │  write)   │ │  decode) │
           └────────────┘ └───────────┘ └──────────┘
                    │
                    ↓
           ┌──────────────┐
           │ bedrock-ffi   │
           │ 46 extern "C" │
           └──────────────┘
```

## Crate 依赖关系

```
bedrock_common                  (zero deps)
    ↑
bedrock-binary-stream           bedrock-nbt
(depends on bedrock_common)     (pure Rust, no deps)
    ↑                               ↑
bedrock-protocol-schema         bedrock-protocol-data
(field types, Schema,           (version blobs, bridge
 ProtocolRegistry,               between schema and
 EmbeddedVersion)                binary data)
    ↑                               ↑
bedrock-codegen ──────────→  bedrock-protocol
(generated codegen,           (Packet trait, types,
 build.rs integration)         NBT, FFI bridge)
                                    ↑
                              bedrock-ffi
                              (46 C API functions)
```

---

## 协议数据生成

### 原理

通过EndstoneMC提供的`protocol-docs` 仓库，得到以 JSON 格式储存的Minecraft Bedrock 协议定义。
每个协议版本对应一个 Git 分支（如 `r26_u2` 对应协议 975），包含：

```
protocol-docs/
├── packets/   ← 190 个 JSON 文件，每个文件定义一个数据包的结构
├── enums/     ← 108 个 JSON 文件，枚举定义
└── types/     ← 196 个 JSON 文件，复合类型定义
```

**工作流**：

```
protocol-docs Git 分支
    │ git checkout r26_u2
    ↓
generate-data 工具
    │ 解析 JSON → 打包为 EmbeddedVersion
    │ 输出为单个 data/v{N}.json 文件
    ↓
bedrock-protocol-data/src/lib.rs
    │ include_str!("../data/v{897,924,944,975,1001}.json")
    │ 编译时嵌入二进制
    ↓
ProtocolRegistry::from_embedded()
    │ 反序列化 → Schema 对象
    ↓
运行时查询：get_packet_by_id() / get_enum_by_name() / is_type()
```

### 添加新协议版本

```bash
# Step 1: 检出目标分支
git clone https://github.com/EndstoneMC/protocol-docs.git /tmp/protocol-docs
cd /tmp/protocol-docs && git checkout r26_u3

# Step 2: 生成数据文件
cd /path/to/protocol
cargo run -p bedrock-protocol-data --bin generate-data -- \
  --docs /tmp/protocol-docs \
  --output ./crates/bedrock-protocol-data/data/v1001.json

# Step 3: 注册到 lib.rs
# 编辑 crates/bedrock-protocol-data/src/lib.rs，在 blobs 数组中添加：
#   include_str!("../data/v1001.json"),

# Step 4: 验证
python3 -c "
import json
d = json.load(open('crates/bedrock-protocol-data/data/v1001.json'))
print(f'v{d[\"network_version\"]}, {len(d[\"packets\"])} packets')
"
cargo test -p bedrock-codegen -- e2e_tests
```

---

## 快速开始

```bash
# 构建全部 crate（默认启用代码生成）
cargo build

# 运行全部测试
cargo test --workspace

# 构建 C 共享库
cargo build -p bedrock-ffi --release
# → target/release/libbedrock_ffi.so / .dylib / .dll
```

---

## License

Apache-2.0
