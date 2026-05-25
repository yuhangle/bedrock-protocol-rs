# bedrock-binary-stream

Bedrock 协议二进制流序列化核心。

## 流类型

| 类型 | 说明 |
|---|---|
| `ReadOnlyBinaryStream` | 只读流，封装持有的 `Vec<u8>` 缓冲区。跟踪读取位置和溢出标志。 |
| `BinaryStream` | 可写流，持有内部 `Vec<u8>` 缓冲区。实现 `BedrockWrite`。 |

## 支持的类型

| 类别 | 类型 |
|---|---|
| **整数** | u8, i16/u16, i32/u32, i64/u64, i32_be, u24 |
| **变长整数** | varint32, uvarint32, varint64, uvarint64 |
| **浮点数** | f32, f64, normalized_f32 |
| **字符串** | string (varint 长度前缀), short_string (u16 长度前缀), long_string (u32 长度前缀) |
| **字节** | raw_bytes |
