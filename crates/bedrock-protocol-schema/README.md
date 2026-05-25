# bedrock-protocol-schema

解析 Minecraft Bedrock Edition 的 `protocol-docs` JSON 协议文档。
提供 Schema 查询 API、嵌入式数据打包、多版本注册表。

## 核心类型

### Schema

```rust
let schema = Schema::load_all(Path::new("./docs/protocol-docs"))?;
// 或从预生成数据加载：
schema = EmbeddedVersion::from_directory(path)?.to_schema()?;

schema.get_packet_by_id(1);             // → Option<&PacketDefinition>
schema.get_packet_by_name("LoginPacket");
schema.get_enum_by_name("Connection::DisconnectFailReason");
schema.get_type_by_name("Vec3");
schema.is_enum("MinecraftPacketIds");   // bool
schema.is_type("BlockPos");             // bool
schema.packet_count();                  // 190
```

### FieldType 字段类型系统

每个字段都有一个 `FieldType`，描述其数据类型：

```rust
pub enum FieldType {
    Named(String),          // 基本类型或类型引用: "bool", "uvarint32", "Vec3", "BlockPos"
    SwitchCase(SwitchCase), // 条件分支，根据 discriminator 值选择不同结构
    Map { key, value },     // 字典类型: {"key": "string", "value": "int32"}
}
```

#### Named
最简单的类型：一个名字。可以是：
- **基本类型**：`bool`, `uint8`, `int32`, `float`, `string`, `varint32`, `uvarint32` 等
- **类型引用**：`Vec3`, `BlockPos`, `ActorUniqueID` 等（在 types/ 目录中定义）
- **枚举引用**：字段的 `enum_ref` 字段指定枚举名（在 enums/ 目录中定义）

#### SwitchCase
条件分支，类似 Rust 的 `enum` 或 `Option`：

```json
{
  "switch": {"type": "uvarint32"},
  "cases": [null, "DisconnectPacketMessages"]
}
```

- 分支0 (`None`) = 无数据
- 分支1 (`Some`) = `DisconnectPacketMessages` 结构体

常用于可选字段、多态类型。codegen 生成 `Option<T>` 或自定义枚举。

```rust
pub struct SwitchCase {
    pub switch_type: Box<FieldType>,    // discriminator 类型（通常 uvarint32）
    pub switch_enum: Option<String>,     // 可选：discriminator 的枚举引用
    pub switch_name: Option<String>,    // 可选：discriminator 字段名
    pub cases: Vec<SwitchCaseBranch>,
}

pub enum SwitchCaseBranch {
    Empty,          // null — 无数据
    Primitive(String), // 内联基本类型: "bool", "int", "float"
    Type(String),   // 类型引用
}
```

#### Map
字典类型，key 和 value 都是类型名或基本类型。

### RepeatInfo

字段可以重复（列表），有两种方式：

```rust
pub struct RepeatInfo {
    pub prefix: Option<String>,  // 变长前缀: "uvarint32"（读取时先读长度）
    pub count: Option<u32>,      // 固定数量: 9（读取固定次数的元素）
}
```

### 嵌入式数据（EmbeddedVersion）

```rust
// 构建时：从 protocol-docs 目录生成
let ev = EmbeddedVersion::from_directory(path)?;
let json = serde_json::to_string(&ev)?;

// 运行时：反序列化回 Schema
let ev: EmbeddedVersion = serde_json::from_str(json_blob)?;
let schema = ev.to_schema()?;
```

### ProtocolRegistry

多版本注册表：

```rust
let registry = ProtocolRegistry::from_embedded(&[blob_v944, blob_v975])?;
registry.get(944);           // → Option<&Schema>
registry.latest();           // → &Schema（自动选最大版本号）
registry.all_versions();     // → 迭代器
registry.has_version(975);   // → bool
registry.version_count();    // → 2
```
