# bedrock-protocol

Minecraft Bedrock Edition 协议定义库。包含 Packet trait、手写类型、
编译时嵌入的协议版本注册表、以及默认启用的代码生成。

---

## Packet trait

```rust
pub trait Packet: BedrockSerializable {
    fn packet_id(&self) -> MinecraftPacketIds;
    fn packet_name(&self) -> &'static str;

    fn serialize(&self) -> Result<Vec<u8>, BinaryStreamError>;
    fn deserialize(&mut self, data: &[u8]) -> Result<(), BinaryStreamError>;
}
```

### UnimplementedPacket

未知数据包 ID 的兜底实现，保留原始字节：
```rust
let pkt = UnimplementedPacket::new(999);
let bytes = pkt.serialize()?; // 透传原始数据
```

---

## MinecraftPacketIds

```rust
// 329 个数据包 ID，范围 0-329
let id = MinecraftPacketIds::Login;          // = 1
let id = MinecraftPacketIds::try_from(1);    // Ok(Login)
let raw: i32 = id.into();                    // 1
```

默认从 `bedrock-protocol-data` 的嵌入协议数据自动生成。
使用 `cargo build --no-default-features` 降级为手写精简版本。

---

## 手写类型

| 类型 | 说明 | 序列化格式 |
|---|---|---|
| `Vec3` | 3D 浮点向量 | 3 × f32 LE |
| `BlockPos` | 方块坐标 | 3 × varint32 |
| `Uuid` | Minecraft UUID | 2 × u64 LE |
| `FullContainerName` | 容器名称 + 动态槽位 | u8 + bool + Option\<u32\> |
| `ItemData` | 物品定义 | string + i16 + bool + varint + Vec\<u8\> |
| `ItemStackRequest` | 物品堆叠请求（复杂嵌套） | 详见 types/item_stack_request.rs |

手写类型在 `src/types/` 下，每个类型一个 `.rs` 文件。如需添加新手写类型，
需要在 `bedrock-codegen/src/gen/types.rs` 的 `is_hand_implemented()` 白名单中注册。

---

## 协议版本注册表

通过 `bedrock-protocol-data` crate 获取编译时嵌入的协议版本：

```rust
use bedrock_protocol_data::registry;

// 获取所有嵌入版本的注册表
let reg = registry();
let v975 = reg.get(975).unwrap();
let latest = reg.latest();
```

详见 `bedrock-protocol-data` crate 的文档和 `MAINTENANCE.md` 的"添加新协议版本"章节。

---

## 代码生成（默认启用）

build.rs 从 `bedrock-protocol-data` 的嵌入协议数据生成 Rust 代码：

```
OUT_DIR/generated.rs  (通过 include! 引入)
├── enums.rs          108 枚举 + TryFrom<i32> + Default
├── types.rs          150+ 类型结构体
├── packets.rs        190 数据包结构体 + Packet + BedrockSerializable
├── packet_ids.rs     MinecraftPacketIds (329 值)
├── factory.rs        MinecraftPackets::create_packet()
└── stubs.rs          未定义类型的 newtype 桩
```

生成的代码与手写类型并存：`crate::types` 下的手写实现优先级高于生成版本。
codegen 通过 `is_hand_implemented()` 白名单自动跳过手写类型。

关闭代码生成：

```bash
cargo build -p bedrock-protocol --no-default-features
```
