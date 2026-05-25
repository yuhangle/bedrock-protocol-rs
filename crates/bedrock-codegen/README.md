# bedrock-codegen

从 Bedrock 协议 schema 生成 Rust 代码。在 `build.rs` 中使用。

## 完整类型映射表

JSON schema 中的类型名 → Rust 代码生成规则：

| JSON 类型 | Rust 类型 | read 调用 | write 调用 |
|---|---|---|---|
| `bool` | `bool` | `read_bool()` | `write_bool(v)` |
| `uint8` / `byte` | `u8` | `read_u8()` | `write_u8(v)` |
| `int16` | `i16` | `read_i16()` | `write_i16(v)` |
| `uint16` / `short` | `u16` | `read_u16()` | `write_u16(v)` |
| `int32` / `int` | `i32` | `read_i32()` | `write_i32(v)` |
| `uint32` | `u32` | `read_u32()` | `write_u32(v)` |
| `int64` | `i64` | `read_i64()` | `write_i64(v)` |
| `uint64` | `u64` | `read_u64()` | `write_u64(v)` |
| `int32_be` | `i32` | `read_i32_be()` | `write_i32_be(v)` |
| `float` | `f32` | `read_f32()` | `write_f32(v)` |
| `double` | `f64` | `read_f64()` | `write_f64(v)` |
| `varint32` / `varint` | `i32` | `read_varint()` | `write_varint(v)` |
| `uvarint32` | `u32` | `read_unsigned_varint()` | `write_unsigned_varint(v)` |
| `varint64` | `i64` | `read_varint64()` | `write_varint64(v)` |
| `uvarint64` | `u64` | `read_unsigned_varint64()` | `write_unsigned_varint64(v)` |
| `normalized_float` | `f32` | `read_normalized_f32()` | `write_normalized_f32(v)` |
| `string` | `String` | `read_string()` | `write_string(&v)` |
| `u24` | `u32` | `read_u24()` | `write_u24(v)` |
| `CompoundTag` | `Vec<u8>` | `read_remaining()` | `write_raw_bytes(&v)` |
| `ActorUniqueID` | `i64` | `read_varint64()` | `write_varint64(v)` |
| `TypeName` | `TypeName` | `TypeName::read_from(stream)` | `val.write_to(stream)` |
| 枚举字段 | `EnumName` | `read_varint()` | `write_varint(v as i32)` |

## 名称转换规则

```
"Client Network Version"  →  client_network_version
"ActorUniqueID"           →  actor_unique_id
"Connection::DisconnectFailReason"  →  ConnectionDisconnectFailReason
"mce::UUID"               →  Uuid（特殊映射）
```

自动处理 Rust 关键字：`type` → `r#type`、`match` → `r#match`。

## 生成的文件

| 文件 | 行数（v975） | 内容 |
|---|---|---|
| `packets.rs` | ~8,000 | 190 个数据包结构体 + Default + BedrockSerializable |
| `factory.rs` | ~200 | MinecraftPackets::create_packet() 工厂 |
| `stubs.rs` | ~300 | 未定义类型的 newtype 桩代码 |

## build.rs 集成

```rust
fn main() {
    let reg = bedrock_protocol_data::registry();
    let schema = reg.get(975).unwrap();  // 或 reg.latest()
    bedrock_codegen::generate_all(schema, &out_dir).expect("codegen failed");
}
```

数据源为 `bedrock-protocol-data` 编译时嵌入的协议版本 JSON，无需外部文件。默认启用（`default = ["generated"]`），使用 `--no-default-features` 关闭。
