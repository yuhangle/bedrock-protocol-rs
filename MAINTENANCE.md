# bedrock-protocol-rs — 维护手册

---

## 目录

1. [项目结构](#1-项目结构)
2. [协议数据生成](#2-协议数据生成)
3. [代码生成](#3-代码生成)
4. [添加新协议版本](#4-添加新协议版本)
5. [FFI 维护](#5-ffi-维护)
6. [测试](#6-测试)
7. [常见陷阱与排查](#7-常见陷阱与排查)

---

## 1. 项目结构

```
protocol/
├── Cargo.toml                          # workspace root，定义 8 个 member crate
├── README.md                           # 项目入口文档
├── MAINTENANCE.md                      # ← 本文档
│
├── crates/
│   ├── bedrock-common/                  # 共享基础层
│   │   ├── error.rs                    #   BinaryStreamError（4 种变体）
│   │   ├── traits.rs                   #   BedrockRead / BedrockWrite / BedrockSerializable
│   │   └── varint.rs                   #   Varint 编解码（4 对 encode/decode 函数）
│   │
│   ├── bedrock-binary-stream/          # 二进制流读写
│   │   ├── read.rs                     #   ReadOnlyBinaryStream（只读，24 个读方法）
│   │   └── write.rs                    #   BinaryStream（可写，24 个写方法）
│   │
│   ├── bedrock-nbt/                    # NBT 编解码
│   │   ├── tag.rs                      #   Tag enum（13 种 NBT 类型）
│   │   ├── compound.rs                 #   CompoundTag（25+ 方法）
│   │   ├── list.rs                     #   ListTag
│   │   ├── decode.rs                   #   解码器（Network NBT + LE/BE + header）
│   │   ├── encode.rs                   #   编码器（4 种 NBT 格式）
│   │   └── snbt.rs                     #   SNBT 解析器（递归下降）
│   │
│   ├── bedrock-protocol-schema/        # 协议 schema 解析与查询
│   │   ├── field.rs                    #   FieldType / RepeatInfo / SwitchCase 定义
│   │   ├── schema.rs                   #   Schema — 协议定义的运行时查询
│   │   ├── embed.rs                    #   EmbeddedVersion — 协议数据的打包/解包
│   │   ├── registry.rs                 #   ProtocolRegistry — 多版本注册表
│   │   └── ...                         #   解析器、验证器等
│   │
│   ├── bedrock-codegen/                # 代码生成器
│   │   ├── gen/
│   │   │   ├── enums.rs               #   枚举代码生成
│   │   │   ├── types.rs               #   类型代码生成
│   │   │   ├── packets.rs             #   数据包代码生成
│   │   │   └── packet_ids.rs          #   MinecraftPacketIds 生成
│   │   ├── naming.rs                   #   命名转换（snake_case, PascalCase）
│   │   └── type_mapping.rs             #   JSON 类型名 → Rust 类型映射
│   │
│   ├── bedrock-protocol/               # 协议库（核心产出）
│   │   ├── lib.rs                      #   Packet trait + 条件导出
│   │   ├── packet.rs                   #   Packet trait 定义
│   │   ├── types/                      #   手写类型
│   │   │   ├── block_pos.rs           #     BlockPos
│   │   │   ├── vec3.rs                #     Vec3
│   │   │   ├── uuid.rs                #     mce::UUID
│   │   │   ├── item_data.rs           #     ItemData
│   │   │   ├── item_stack_request.rs  #     ItemStackRequest(Data/Action/...)
│   │   │   └── full_container_name.rs #     FullContainerName
│   │   ├── generated.rs               #   条件编译，引入生成代码
│   │   └── build.rs                    #   默认启用的代码生成
│   │
│   ├── bedrock-protocol-data/          # 预生成协议版本数据
│   │   ├── src/lib.rs                  #   registry() 加载所有嵌入版本
│   │   ├── src/bin/generate-data.rs   #   generate-data CLI 工具
│   │   └── data/
	│       ├── v897.json              #     r21_u13  (1.21.130.28)
	│       ├── v924.json              #     r26_u0   (1.26.0.29)
	│       ├── v944.json              #     r26_u1   (1.26.10.27)
	│       ├── v975.json              #     r26_u2   (1.26.20.28) ← current
	│       └── v1001.json             #     r26_u3   (1.26.30.30)
│   │
│   └── bedrock-ffi/                    # C FFI
│       └── src/lib.rs                  #   46 个 extern "C" 函数
│                                       #   流(26) + 数据包(6) + NBT(18)
```

### 依赖关系图

```
bedrock_common       bedrock-nbt
    ↑                   ↑
bedrock-binary-stream    │
    ↑                    │
bedrock-protocol-schema  │
    ↑           ↑        │
bedrock-codegen → bedrock-protocol
                       ↑
                  bedrock-ffi
```

### Crate 职责速览

| Crate | 作用 | 关键类型 | 对外依赖 |
|---|---|---|---|
| bedrock_common | 共享基础：错误、trait、varint | BinaryStreamError, BedrockSerializable | 无 |
| bedrock-binary-stream | 二进制流读写核心 | ReadOnlyBinaryStream, BinaryStream | bedrock_common |
| bedrock-nbt | NBT 编解码 | CompoundTag, ListTag, Tag | 无 |
| bedrock-protocol-schema | protocol-docs JSON 解析 + 注册表 | Schema, ProtocolRegistry | bedrock_common |
| bedrock-codegen | 从 Schema 生成 Rust 代码 | — | bedrock-protocol-schema |
| bedrock-protocol | 协议定义库 | Packet trait, 手写类型, 版本注册表 | bedrock_common, bedrock-nbt |
| bedrock-protocol-data | 编译时嵌入的协议版本数据 | — | bedrock-protocol-schema |
| bedrock-ffi | C API 导出 | 46 个 extern "C" | bedrock-protocol, bedrock-nbt |

---

## 2. 协议数据生成

### 2.1 原理

Minecraft Bedrock 协议定义由 EndstoneMC 维护在 [protocol-docs](https://github.com/EndstoneMC/protocol-docs) 仓库中。
每个协议版本是一个 Git 分支（如 `r26_u2` = 协议 975），包含三类 JSON 文件：

| 目录 | 数量（v975） | 内容 |
|---|---|---|
| `packets/` | 190 | 数据包结构定义：字段名、类型、顺序 |
| `enums/` | 108 | 枚举定义：名称、值 |
| `types/` | 196 | 复合类型定义：结构体 |

每个 JSON 文件的格式示例：

```json
// packets/LoginPacket.json
{
  "id": 1,
  "name": "LoginPacket",
  "fields": [
    {"name": "Protocol Version", "type": "int32_be"},
    {"name": "Connection Request", "type": "ConnectionRequest"}
  ]
}
```

### 2.2 数据管道

```
protocol-docs  JSON 文件
    │
    │  bedrock-protocol-schema::Schema::load_all()
    │  递归扫描目录，解析所有 JSON
    │  构建类型引用关系图
    ↓
Schema 对象（内存中）
    │
    │  bedrock-protocol-schema::EmbeddedVersion::from_schema()
    │  序列化为 JSON blob（扁平化，无外部引用）
    ↓
data/v{N}.json（单个文件，自包含）
    │
    │  include_str!("../data/v975.json") 编译时嵌入
    ↓
ProtocolRegistry::from_embedded()
    │  反序列化 → Schema 对象
    ↓
运行时查询
```

### 2.3 EmbeddedVersion 格式

`EmbeddedVersion` 是协议数据的**序列化中间格式**，将 Schema 的图结构扁平化为可嵌入的 JSON：

```rust
pub struct EmbeddedVersion {
    pub network_version: u32,       // 协议版本号，如 975
    pub minecraft_version: String,  // Minecraft 版本，如 "1.21.50"
    pub branch_name: String,        // Git 分支名，如 "r26_u2"
    pub packets: Vec<PacketDef>,    // 扁平化的数据包定义列表
    pub enums: Vec<EnumDef>,        // 扁平化的枚举定义列表
    pub types: Vec<TypeDef>,        // 扁平化的类型定义列表
}
```

该结构由 `generate-data` 工具从 protocol-docs 目录生成，输出为一个 JSON 文件。

---

## 3. 代码生成

### 3.1 build.rs 集成

`bedrock-protocol/build.rs` 在**每次构建时**执行代码生成，数据源为 `bedrock-protocol-data` 编译时嵌入的协议版本 JSON：

```
bedrock-protocol-data 嵌入的 JSON
    │ bedrock_protocol_data::registry().latest()
    ↓
bedrock_codegen::generate_all(&schema, &out_dir)
    ↓
OUT_DIR/generated.rs  (通过 include! 引入)
├── enums.rs         108 枚举 + TryFrom<i32> + Default
├── types.rs         150+ 类型结构体
├── packets.rs       190 数据包结构体 + Packet + BedrockSerializable
├── packet_ids.rs    MinecraftPacketIds (329 值)
├── factory.rs       MinecraftPackets::create_packet()
└── stubs.rs         未定义类型的 newtype 桩
```

代码生成默认启用（`default = ["generated"]`）。如需关闭（使用手写 IDs + `UnimplementedPacket`），执行：

```bash
cargo build --no-default-features
```

调试时检查 codegen 输出：

```bash
grep "cargo:info=Generated" target/debug/build/bedrock-protocol-*/output
```

### 3.2 手写类型与生成代码的交互

手写类型放在 `bedrock-protocol/src/types/`，由 `types/mod.rs` 统一导出。
codegen 通过 `is_hand_implemented()` 白名单跳过：

```rust
// bedrock-codegen/src/gen/types.rs
fn is_hand_implemented(name: &str) -> bool {
    matches!(name,
        "Vec3" | "BlockPos" | "FullContainerName" | "ItemData"
        | "ItemStackRequest" | "ItemStackRequestAction" | ...
    )
}
```

**添加新手写类型**的步骤：
1. 在 `type/` 下添加 `.rs` 文件
2. 在 `types/mod.rs` 中 `pub use`
3. 在 codegen 白名单中加入类型名

### 3.3 类型映射规则

| JSON 类型 | Rust 类型 | read 方法 | write 方法 |
|---|---|---|---|
| `bool` | `bool` | `read_bool()` | `write_bool(v)` |
| `uint8` / `byte` | `u8` | `read_u8()` | `write_u8(v)` |
| `int16` | `i16` | `read_i16()` | `write_i16(v)` |
| `int32` / `int` | `i32` | `read_i32()` | `write_i32(v)` |
| `varint32` / `varint` | `i32` | `read_varint()` | `write_varint(v)` |
| `string` | `String` | `read_string()` | `write_string(&v)` |
| `CompoundTag` | `Vec<u8>` | `read_remaining()` | `write_raw_bytes(&v)` |
| 枚举引用 | `EnumName` | `<Enum>::try_from(read_varint())` | `write_varint(v as i32)` |
| 类型引用 | `TypeName` | `TypeName::read_from(stream)` | `val.write_to(stream)` |

### 3.4 生成失败类型的降级

当类型引用无法解析（如非法名称、模板语法、复杂 SwitchCase）时，codegen 自动降级为 `Vec<u8>` 桩：

```rust
// 生成代码中的降级示例
pub struct UnresolvedType(pub Vec<u8>);

impl BedrockSerializable for UnresolvedType {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError> {
        stream.write_raw_bytes(&self.0)
    }
    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError> {
        Ok(Self(stream.read_remaining()?))
    }
}
```

---

## 4. 添加新协议版本

### 完整工作流

```bash
# Step 1: 获取 protocol-docs 目标分支
git clone https://github.com/EndstoneMC/protocol-docs.git /tmp/protocol-docs
cd /tmp/protocol-docs && git checkout r26_u3
cd /path/to/protocol

# Step 2: 生成数据文件
cargo run -p bedrock-protocol-data --bin generate-data -- \
  --docs /tmp/protocol-docs \
  --output ./crates/bedrock-protocol-data/data/v1001.json

# Step 3: 验证数据
python3 -c "
import json
d = json.load(open('crates/bedrock-protocol-data/data/v1001.json'))
print(f'v{d[\"network_version\"]} ({d[\"branch_name\"]}), MC {d[\"minecraft_version\"]}')
print(f'  {len(d[\"packets\"])} packets, {len(d[\"enums\"])} enums, {len(d[\"types\"])} types')
"

# Step 4: 注册到 lib.rs
# 编辑 crates/bedrock-protocol-data/src/lib.rs，在 blobs 数组中：
#   include_str!("../data/v1001.json"),

# Step 5: 提交
git add crates/bedrock-protocol-data/data/v1001.json
git commit -m "add protocol v1001 data"
```

### 版本枚举生成

数据包 ID 枚举 `MinecraftPacketIds` 从 `protocol-docs` 的共享定义自动生成。
添加新版本后需运行测试确认：

```bash
cargo test -p bedrock-codegen -- e2e_tests
```

---

## 5. FFI 维护

### C API 概览

46 个 extern "C" 函数，分组如下：

| 类别 | 数量 | 前缀 | 说明 |
|---|---|---|---|
| 流生命周期 | 3 | `bedrock_stream_create/destroy/from_bytes` | 创建/销毁二进制流 |
| 流读取 | 19 | `bedrock_stream_read_*` | 只读流读操作 |
| 流写入 | 16 | `bedrock_stream_write_*` | 可写流写操作 |
| 流工具 | 4 | `bedrock_stream_size/position/set_position/data` | 位置查询、内部缓冲区 |
| 数据包 | 6 | `bedrock_packet_*` | 数据包创建/序列化 |
| NBT | 18 | `bedrock_nbt_*` | NBT 操作 |
| 内存 | 1 | `bedrock_free` | 释放 Rust 分配的缓冲区 |
| 错误 | 1 | `bedrock_last_error` | 获取错误信息 |

### 关键约束

**错误码**：所有函数返回 `i32`，0=成功，负数=错误。

| 常量 | 值 | 含义 | 常见原因 |
|---|---|---|---|
| `BEDROCK_ERR_OVERFLOW` | -1 | 缓冲区溢出 | 读取超过流末尾 |
| `BEDROCK_ERR_INVALID_DATA` | -2 | 数据无效 | varint 不完整、UTF-8 非法 |
| `BEDROCK_ERR_INVALID_ARG` | -3 | 参数无效 | null 指针、只读流上写入 |
| `BEDROCK_ERR_UNSUPPORTED` | -4 | 不支持的操作 | 内存分配失败 |
| `BEDROCK_ERR_NBT` | -5 | NBT 错误 | NBT 解析失败 |

错误时调用 `bedrock_last_error()` 获取可读消息（线程安全）。

**内存管理**：`bedrock_packet_serialize`、`bedrock_nbt_to_binary` 等函数返回的内存
必须用 `bedrock_free` 释放（内部使用 `libc::malloc`）。

### Packet trait 的 FFI 限制

`Packet::deserialize` 使用了 `where Self: Sized`，因此不能通过 `dyn Packet` 调用。
FFI 代码直接操作具体类型（`UnimplementedPacket`），不受影响。

```rust
// Packet trait 定义
pub trait Packet: BedrockSerializable {
    fn packet_id(&self) -> MinecraftPacketIds;
    fn packet_name(&self) -> &'static str;
    fn serialize(&self) -> Result<Vec<u8>, BinaryStreamError>;
    fn deserialize(&mut self, data: &[u8]) -> Result<(), BinaryStreamError>
    where Self: Sized;
}
```

工厂函数通过 `Box<dyn Packet>` 分发，但仅调用 `serialize`/`packet_id`/`packet_name`，
这些方法不要求 `Sized`。反序列化在具体类型上调用。

---

## 6. 测试

```
cargo test --workspace                # 全量测试（108+ 个）
cargo test -p bedrock-binary-stream   # 42 个（含 C++ hex 兼容性测试）
cargo test -p bedrock-protocol-schema # 5 个
cargo test -p bedrock-codegen         # 17 个
cargo test -p bedrock-nbt             # 25 个（含 RapidNBT 兼容 + SNBT）
cargo test -p bedrock-protocol        # 2 个
```

测试分为几类：

| 类别 | 位置 | 说明 |
|---|---|---|
| 兼容性测试 | bedrock-binary-stream | 与 C++ bstream 的 hex 输出逐字节对比 |
| 兼容性测试 | bedrock-nbt | 与 RapidNBT 的序列化输出逐字节对比 |
| Roundtrip | bedrock-binary-stream | 写入 → 读取 → 值一致 |
| Roundtrip | bedrock-nbt | 编码 → 解码 → 值一致；SNBT 往返 |
| 属性测试 | bedrock_common | varint 编解码的数学性质验证 |
| 代码生成 | bedrock-codegen | E2E：从 protocol-docs 生成 → 编译通过 |
| Schema 解析 | bedrock-protocol-schema | JSON 加载、查询、嵌入数据 roundtrip |

### 验证十六进制输出的标准模式

```rust
let expected = "0102030004000000...";
let got = hex::encode(stream.into_data());
assert_eq!(got, expected, "hex mismatch");
```

---

## 7. 常见陷阱与排查

### 7.1 枚举值重复（E0081）

C++ 协议允许枚举有重复值，但 Rust 不允许。

**症状**：`error[E0081]: discriminant value already used`

**修复位置**：`bedrock-codegen/src/gen/enums.rs` 中 `generate_enum` 函数——按 value 去重。

### 7.2 字段名含特殊字符

JSON 字段名可能含 `()`, `[]`, `,:;` 等 Rust 非法字符。

**症状**：编译错误 `expected `(`, found `:`

**修复位置**：`bedrock-codegen/src/naming.rs` 中 `to_snake_case`——补充字符过滤。

### 7.3 类型不在作用域

生成代码引用了 schema 中的类型，但该类型在生成时被跳过（非法名称等）。

**症状**：`cannot find type Xxx`

**修复位置**：`bedrock-codegen/src/gen/packets.rs` 中 `inner_type`——检查 schema 类型 / 非法名称回落为 `Vec<u8>`。

### 7.4 协议数据缺失

`build.rs` 从 `bedrock-protocol-data` 的嵌入 JSON 读取数据。

**症状**：编译错误提示找不到 `generated.rs`。

**排查**：
```bash
# 确认协议数据文件是否存在
ls crates/bedrock-protocol-data/data/

# 检查 codegen 是否正确执行
grep "cargo:info=Generated" target/debug/build/bedrock-protocol-*/output
```

### 7.6 NBT 编码兼容性

Bedrock Network NBT 格式中，各类型的编码方式与 LE/BE 格式不同：

| 类型 | Network NBT 编码 | LE/BE NBT 编码 |
|---|---|---|
| Byte | 1 字节 | 1 字节 |
| Short | **2 字节 LE**（非 ZigZag） | 2 字节 LE/BE |
| Int | **ZigZag varint32** | 4 字节 LE/BE |
| Long | **ZigZag varint64** | 8 字节 LE/BE |
| Float/Double | 4/8 字节 LE | 4/8 字节 LE/BE |
| IntArray 元素 | **ZigZag varint32** 每个 | 4 字节 LE/BE 每个 |

### 7.7 枚举生成的 Default 实现

有 0 变体 → 用 `try_from(0).ok().unwrap()`。无 0 变体 → `unsafe { std::mem::zeroed() }`。

### 7.8 命名转换规则

```
"Client Network Version"  →  client_network_version
"ActorUniqueID"           →  actor_unique_id
"Connection::DisconnectFailReason"  →  ConnectionDisconnectFailReason
"mce::UUID"               →  Uuid（特殊映射）
"type"                    →  r#type（Rust 关键词转义）
```

	