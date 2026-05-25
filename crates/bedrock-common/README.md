# bedrock-common

工具链的共享基础 crate。**几乎所有其他 crate 都依赖此 crate。**

## 组件

### 1. BinaryStreamError

所有流操作的错误类型。

```rust
pub enum BinaryStreamError {
    Overflow { position: usize, size: usize },     // 读写超出缓冲区
    InvalidData { description: &'static str },     // 数据格式错误
    UnsupportedValue { description: String },       // 不支持的值
    NbtError { description: String },               // NBT 错误
}
```

实现了 `Display`、`Error`、`From`。所有操作均返回 `Result<_, BinaryStreamError>`，永不 panic。

### 2. BedrockSerializable trait

所有可序列化类型的核心 trait：

```rust
pub trait BedrockSerializable {
    fn write_to(&self, stream: &mut dyn BedrockWrite) -> Result<(), BinaryStreamError>;
    fn read_from(stream: &mut dyn BedrockRead) -> Result<Self, BinaryStreamError>
    where Self: Sized;
}
```

**对称模式**：`write_to` 按协议顺序写入字段，`read_from` 按相同顺序读取。
所有数据包和嵌套类型都实现此 trait。

### 3. BedrockRead trait（24 个方法）

```rust
pub trait BedrockRead {
    fn read_bool(&mut self) -> Result<bool, BinaryStreamError>;
    fn read_u8(&mut self) -> Result<u8, BinaryStreamError>;
    fn read_i16(&mut self) -> Result<i16, BinaryStreamError>;
    fn read_u16(&mut self) -> Result<u16, BinaryStreamError>;
    fn read_i32(&mut self) -> Result<i32, BinaryStreamError>;
    fn read_u32(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_i64(&mut self) -> Result<i64, BinaryStreamError>;
    fn read_u64(&mut self) -> Result<u64, BinaryStreamError>;
    fn read_f32(&mut self) -> Result<f32, BinaryStreamError>;
    fn read_f64(&mut self) -> Result<f64, BinaryStreamError>;
    fn read_i32_be(&mut self) -> Result<i32, BinaryStreamError>;       // 大端
    fn read_u32_be(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_u24(&mut self) -> Result<u32, BinaryStreamError>;           // 24位 LE
    fn read_varint(&mut self) -> Result<i32, BinaryStreamError>;        // ZigZag
    fn read_varint64(&mut self) -> Result<i64, BinaryStreamError>;
    fn read_unsigned_varint(&mut self) -> Result<u32, BinaryStreamError>;
    fn read_unsigned_varint64(&mut self) -> Result<u64, BinaryStreamError>;
    fn read_normalized_f32(&mut self) -> Result<f32, BinaryStreamError>;
    fn read_string(&mut self) -> Result<String, BinaryStreamError>;     // varint长度前缀
    fn read_short_string(&mut self) -> Result<String, BinaryStreamError>; // u16长度前缀
    fn read_long_string(&mut self) -> Result<String, BinaryStreamError>;  // u32长度前缀
    fn read_raw_bytes(&mut self, len: usize) -> Result<Vec<u8>, BinaryStreamError>;
    fn read_remaining(&mut self) -> Result<Vec<u8>, BinaryStreamError>;
}
```

### 4. BedrockWrite trait（24 个方法）

```rust
pub trait BedrockWrite {
    fn write_bool(&mut self, value: bool) -> Result<(), BinaryStreamError>;
    fn write_u8(&mut self, value: u8) -> Result<(), BinaryStreamError>;
    fn write_i16(&mut self, value: i16) -> Result<(), BinaryStreamError>;
    fn write_u16(&mut self, value: u16) -> Result<(), BinaryStreamError>;
    fn write_i32(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_u32(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_i64(&mut self, value: i64) -> Result<(), BinaryStreamError>;
    fn write_u64(&mut self, value: u64) -> Result<(), BinaryStreamError>;
    fn write_f32(&mut self, value: f32) -> Result<(), BinaryStreamError>;
    fn write_f64(&mut self, value: f64) -> Result<(), BinaryStreamError>;
    fn write_i32_be(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_u32_be(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_u24(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_varint(&mut self, value: i32) -> Result<(), BinaryStreamError>;
    fn write_varint64(&mut self, value: i64) -> Result<(), BinaryStreamError>;
    fn write_unsigned_varint(&mut self, value: u32) -> Result<(), BinaryStreamError>;
    fn write_unsigned_varint64(&mut self, value: u64) -> Result<(), BinaryStreamError>;
    fn write_normalized_f32(&mut self, value: f32) -> Result<(), BinaryStreamError>;
    fn write_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_short_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_long_string(&mut self, value: &str) -> Result<(), BinaryStreamError>;
    fn write_raw_bytes(&mut self, value: &[u8]) -> Result<(), BinaryStreamError>;
}
```

### 5. Varint 编解码

所有 varint 使用 Protocol Buffers 风格的 **base-128 变长编码**。
有符号类型使用 **ZigZag 编码**：`(n << 1) ^ (n >> 31)`。

| 函数 | 输入 | 编码方式 | 最大字节 |
|---|---|---|---|
| `encode_unsigned_varint` | u32 | 直接编码 | 5 |
| `encode_varint32` | i32 | ZigZag → u32 → varint | 5 |
| `encode_unsigned_varint64` | u64 | 直接编码 | 10 |
| `encode_varint64` | i64 | ZigZag → u64 → varint | 10 |
| `unsigned_varint_size` | u32 | 计算编码后字节数 | - |
| `varint32_size` | i32 | 计算编码后字节数 | - |

每个编码函数有对应的 `decode_*` 函数，返回 `(值, 消耗字节数)`。
