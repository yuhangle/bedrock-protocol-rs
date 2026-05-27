# bedrock-ffi

C FFI bindings for the Bedrock protocol library. Produces `libbedrock_ffi.so` / `.dylib` / `.dll`.

## 构建

```bash
cargo build -p bedrock-ffi --release
# → target/release/libbedrock_ffi.so
```

## 完整 API 参考

### 流生命周期

```c
// 创建可写流
bedrock_stream_t* s = bedrock_stream_create(0);  // 0 = little-endian, 1 = big-endian

// 从已有数据创建只读流
bedrock_stream_t* s = bedrock_stream_from_bytes(data, len, 0);

// 销毁
void bedrock_stream_destroy(s);
```

### 读取（24 个函数）

全部返回 `int` 错误码。值通过 `out` 指针返回。

```c
int bedrock_stream_read_bool(stream, bool* out);
int bedrock_stream_read_u8(stream, uint8_t* out);
int bedrock_stream_read_i16(stream, int16_t* out);
int bedrock_stream_read_u16(stream, uint16_t* out);
int bedrock_stream_read_i32(stream, int32_t* out);
int bedrock_stream_read_u32(stream, uint32_t* out);
int bedrock_stream_read_i64(stream, int64_t* out);
int bedrock_stream_read_u64(stream, uint64_t* out);
int bedrock_stream_read_f32(stream, float* out);
int bedrock_stream_read_f64(stream, double* out);
int bedrock_stream_read_i32_be(stream, int32_t* out);   // 大端 int32
int bedrock_stream_read_u32_be(stream, uint32_t* out);  // 大端 uint32
int bedrock_stream_read_u24(stream, uint32_t* out);     // 24-bit LE
int bedrock_stream_read_varint(stream, int32_t* out);
int bedrock_stream_read_varint64(stream, int64_t* out);
int bedrock_stream_read_unsigned_varint(stream, uint32_t* out);
int bedrock_stream_read_unsigned_varint64(stream, uint64_t* out);
int bedrock_stream_read_normalized_f32(stream, float* out);

// 字符串：先传 NULL 获取所需长度，再传 buffer
size_t needed;
bedrock_stream_read_string(stream, NULL, &needed);
char* buf = malloc(needed);
bedrock_stream_read_string(stream, buf, &needed);
```

### 写入（20 个函数）

仅在可写流上可用（通过 `bedrock_stream_create` 创建）。

```c
int bedrock_stream_write_bool(stream, bool value);
int bedrock_stream_write_u8(stream, uint8_t value);
int bedrock_stream_write_i16(stream, int16_t value);
int bedrock_stream_write_u16(stream, uint16_t value);
int bedrock_stream_write_i32(stream, int32_t value);
int bedrock_stream_write_u32(stream, uint32_t value);
int bedrock_stream_write_i64(stream, int64_t value);
int bedrock_stream_write_u64(stream, uint64_t value);
int bedrock_stream_write_f32(stream, float value);
int bedrock_stream_write_f64(stream, double value);
int bedrock_stream_write_i32_be(stream, int32_t value);
int bedrock_stream_write_u32_be(stream, uint32_t value);
int bedrock_stream_write_u24(stream, uint32_t value);
int bedrock_stream_write_varint(stream, int32_t value);
int bedrock_stream_write_varint64(stream, int64_t value);
int bedrock_stream_write_unsigned_varint(stream, uint32_t value);
int bedrock_stream_write_unsigned_varint64(stream, uint64_t value);
int bedrock_stream_write_normalized_f32(stream, float value);
int bedrock_stream_write_string(stream, const char* value);
int bedrock_stream_write_raw_bytes(stream, const uint8_t* data, size_t len);
```

### 流工具

```c
size_t bedrock_stream_size(stream);          // 总大小
size_t bedrock_stream_position(stream);      // 当前位置
int    bedrock_stream_set_position(stream, pos);  // 设置位置
int    bedrock_stream_data(stream, &data, &len);  // 获取内部缓冲区指针
```

### 数据包操作

```c
// 创建/销毁
bedrock_packet_t* pkt = bedrock_packet_create(packet_id);
void bedrock_packet_destroy(pkt);

// 序列化（返回的 data 需用 bedrock_free 释放）
uint8_t* data;
size_t len;
bedrock_packet_serialize(pkt, &data, &len);
bedrock_free(data);

// 反序列化
bedrock_packet_deserialize(pkt, bytes, bytes_len);

// 查询
uint32_t     id   = bedrock_packet_get_id(pkt);
const char* name = bedrock_packet_get_name(pkt);
```

### NBT 操作（35 个函数）

```c
// 生命周期
void* nbt = bedrock_nbt_create();
void bedrock_nbt_destroy(nbt);

// 设值
bedrock_nbt_set_string(nbt, "Name", "hello");
bedrock_nbt_set_int(nbt, "RepairCost", 1);
bedrock_nbt_set_short(nbt, "lvl", 5);
bedrock_nbt_set_byte(nbt, "Unbreakable", 1);
bedrock_nbt_set_tag(nbt, "child", child_nbt);  // 嵌入子 CompoundTag（消耗 child 所有权）

// 列表
bedrock_nbt_list_append_string(nbt, "Lore", "line 1");
bedrock_nbt_list_append_tag(nbt, "list", child_nbt);  // 追加 CompoundTag 到列表

// 序列化（返回的 data 需用 bedrock_free 释放）
uint8_t* data; size_t len;
bedrock_nbt_to_binary(nbt, &data, &len);    // Little Endian NBT
bedrock_nbt_to_network(nbt, &data, &len);   // Bedrock Network NBT
bedrock_nbt_to_snbt(nbt);                   // SNBT 字符串（需 bedrock_free）
bedrock_nbt_write_to_stream(nbt, stream);   // 写入 BinaryStream
bedrock_free(data);

// 反序列化（解析到已存在的 nbt 句柄）
bedrock_nbt_from_network_into(nbt, data, len, &consumed);  // Bedrock Network NBT
bedrock_nbt_from_binary_into(nbt, data, len, little_endian, &consumed);  // 标准二进制 NBT

// 查询
bool empty = bedrock_nbt_empty(nbt);
bool has = bedrock_nbt_contains(nbt, "Name");

// 读取（值通过 out 指针返回）
int32_t int_val;
bedrock_nbt_get_int(nbt, "count", &int_val);

int8_t  byte_val;
bedrock_nbt_get_byte(nbt, "byte_key", &byte_val);

int16_t short_val;
bedrock_nbt_get_short(nbt, "short_key", &short_val);

int64_t long_val;
bedrock_nbt_get_long(nbt, "long_key", &long_val);

float  float_val;
bedrock_nbt_get_float(nbt, "float_key", &float_val);

double double_val;
bedrock_nbt_get_double(nbt, "double_key", &double_val);

char buf[256];
size_t buf_len = sizeof(buf);
bedrock_nbt_get_string(nbt, "name", buf, &buf_len);

// 嵌套 CompoundTag（返回新句柄，需 bedrock_nbt_destroy）
void* child = bedrock_nbt_get_tag(nbt, "child_key");

// 数组读取（返回的 data 需 bedrock_free）
uint8_t* ba_data; size_t ba_len;
bedrock_nbt_get_byte_array(nbt, "byte_array_key", &ba_data, &ba_len);
int32_t* ia_data; size_t ia_len;
bedrock_nbt_get_int_array(nbt, "int_array_key", &ia_data, &ia_len);

// 条目枚举
size_t count = bedrock_nbt_entry_count(nbt);
const char* key = bedrock_nbt_entry_key_at(nbt, index);
int type = bedrock_nbt_entry_type_at(nbt, index);  // 0=End, 1=Byte, 2=Short, 3=Int, ...

// 安全 key 复制（避免指针生命周期问题）
char key_buf[256];
size_t key_len = sizeof(key_buf);
bedrock_nbt_entry_key_copy(nbt, index, key_buf, &key_len);

// 列表查询
int list_sz = bedrock_nbt_list_size(nbt, "list_key");
int elem_type = bedrock_nbt_list_get_element_type(nbt, "list_key");
void* elem = bedrock_nbt_list_get_tag_at(nbt, "list_key", index);  // Compound 元素
char elem_buf[256];
size_t elem_len = sizeof(elem_buf);
bedrock_nbt_list_get_string_at(nbt, "list_key", index, elem_buf, &elem_len);  // String 元素
```

---

## 完整示例

### C 示例：写入和读取

```c
#include <stdio.h>
#include <string.h>
#include "bedrock_ffi.h"  // 需要手动生成或参照此 API

int main() {
    // 创建可写流
    bedrock_stream_t* s = bedrock_stream_create(0);
    
    // 写入字段（模仿 Minecraft LoginPacket 的结构）
    bedrock_stream_write_i32_be(s, 975);          // 协议版本（大端）
    bedrock_stream_write_string(s, "{}");         // JWT token（简化）
    
    // 获取数据
    const uint8_t* data;
    size_t len;
    bedrock_stream_data(s, &data, &len);
    printf("Wrote %zu bytes\n", len);
    
    // 创建只读流读取
    bedrock_stream_t* r = bedrock_stream_from_bytes(data, len, 0);
    
    int32_t version;
    char buf[4096];
    size_t buf_len = sizeof(buf);
    
    bedrock_stream_read_i32_be(r, &version);
    bedrock_stream_read_string(r, buf, &buf_len);
    
    printf("Version: %d, Token: %s\n", version, buf);
    
    bedrock_stream_destroy(s);
    bedrock_stream_destroy(r);
    return 0;
}
```

### C++ 示例：流操作（RAII 封装）

```cpp
#include <cstdint>
#include <cstdio>
#include <memory>

struct StreamDeleter {
    void operator()(bedrock_stream_t* s) { bedrock_stream_destroy(s); }
};
using StreamPtr = std::unique_ptr<bedrock_stream_t, StreamDeleter>;

int main() {
    // 写入
    StreamPtr w(bedrock_stream_create(0));
    bedrock_stream_write_varint(w.get(), 42);
    bedrock_stream_write_string(w.get(), "hello");

    // 取出数据
    const uint8_t* data;
    size_t len;
    bedrock_stream_data(w.get(), &data, &len);

    // 从数据创建只读流并读取
    StreamPtr r(bedrock_stream_from_bytes(data, len, 0));
    int32_t val;
    bedrock_stream_read_varint(r.get(), &val);
    printf("Read: %d\n", val);  // 42

    // 错误处理
    int32_t dummy;
    int rc = bedrock_stream_read_varint(r.get(), &dummy);
    if (rc != 0) {
        printf("Error: %s\n", bedrock_last_error());
    }
    return 0;
}
```

### C++ 示例：NBT 操作

```cpp
#include <cstdio>
#include <memory>

struct NbtDeleter {
    void operator()(void* n) { bedrock_nbt_destroy(n); }
};
using NbtPtr = std::unique_ptr<void, NbtDeleter>;

int main() {
    NbtPtr nbt(bedrock_nbt_create());

    bedrock_nbt_set_string(nbt.get(), "Name", "钻石剑");
    bedrock_nbt_set_int(nbt.get(), "RepairCost", 3);

    // 序列化为 SNBT
    char* snbt = bedrock_nbt_to_snbt(nbt.get());
    if (snbt) {
        printf("%s\n", snbt);
        bedrock_free(snbt);
    }

    // 序列化为二进制
    uint8_t* bin;
    size_t bin_len;
    bedrock_nbt_to_binary(nbt.get(), &bin, &bin_len);
    bedrock_free(bin);

    return 0;
}
```

### Python 示例（ctypes）

```python
import ctypes

lib = ctypes.CDLL("./target/release/libbedrock_ffi.so")

# 创建流
lib.bedrock_stream_create.restype = ctypes.c_void_p
stream = lib.bedrock_stream_create(0)  # little-endian

# 写入
lib.bedrock_stream_write_varint(stream, 42)
lib.bedrock_stream_write_string(stream, b"hello".decode())

# 读取
lib.bedrock_stream_set_position(stream, 0)
val = ctypes.c_int32()
lib.bedrock_stream_read_varint(stream, ctypes.byref(val))
print(f"Read: {val.value}")

# 错误处理
rc = lib.bedrock_stream_read_varint(stream, ctypes.byref(val))
if rc != 0:
    lib.bedrock_last_error.restype = ctypes.c_char_p
    print(f"Error: {lib.bedrock_last_error().decode()}")

lib.bedrock_stream_destroy(stream)
```

### Python 示例：数据包序列化

```python
import ctypes

lib = ctypes.CDLL("./target/release/libbedrock_ffi.so")

# 创建 LoginPacket (id=1)
lib.bedrock_packet_create.restype = ctypes.c_void_p
pkt = lib.bedrock_packet_create(1)

# 序列化
out_data = ctypes.POINTER(ctypes.c_uint8)()
out_len = ctypes.c_size_t()
lib.bedrock_packet_serialize(pkt, ctypes.byref(out_data), ctypes.byref(out_len))

# 读取序列化结果
data = bytes(out_data[:out_len.value])
print(f"Serialized {len(data)} bytes: {data.hex()}")

# 释放
lib.bedrock_free(out_data)
lib.bedrock_packet_destroy(pkt)
```

---

## 错误码

| 常量 | 值 | 含义 | 常见原因 |
|---|---|---|---|
| `BEDROCK_SUCCESS` | 0 | 成功 | - |
| `BEDROCK_ERR_OVERFLOW` | -1 | 缓冲区溢出 | 读取超过流末尾 |
| `BEDROCK_ERR_INVALID_DATA` | -2 | 数据无效 | varint 不完整、UTF-8 非法 |
| `BEDROCK_ERR_INVALID_ARG` | -3 | 参数无效 | null 指针、只读流上写入 |
| `BEDROCK_ERR_UNSUPPORTED` | -4 | 不支持的操作 | 内存分配失败 |
| `BEDROCK_ERR_NBT` | -5 | NBT 错误 | NBT 解析失败 |

错误时调用 `const char* bedrock_last_error()` 获取可读消息（线程安全）。

## 设计要点

- 所有函数不 panic，错误返回码 + `bedrock_last_error()` 消息
- 错误消息使用 `std::thread_local` 存储，线程安全
- `bedrock_packet_serialize` 返回的内存必须用 `bedrock_free` 释放（内部用 `libc::malloc`)
- 当前数据包实现基于 `UnimplementedPacket`，支持任意 packet_id
- 读操作和写操作是不同的函数集，不可混用（只读流上写入返回错误）
