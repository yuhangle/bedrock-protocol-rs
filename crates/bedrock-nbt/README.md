# bedrock-nbt

纯 Rust NBT 实现，用于 Minecraft Bedrock Edition。

## 用法

```rust
use bedrock_nbt::{CompoundTag, ListTag};

// 构建 NBT
let mut tag = CompoundTag::new();
tag.set("Name", "hello");
tag.set("RepairCost", 1i32);
tag.set("Unbreakable", 1i8);

let mut lore = ListTag::new();
lore.append("line 1");
lore.append("line 2");
tag.set("Lore", lore);

// 序列化
let binary = tag.to_binary_nbt(true, false);       // Little Endian NBT
let header = tag.to_binary_nbt_with_header(true, None);  // LE + header
let network = tag.to_network_nbt();                 // Bedrock Network NBT
let snbt = tag.to_snbt();                           // → {"Name": "hello", ...}

// 反序列化
let (parsed, _) = CompoundTag::from_network_nbt(&network).unwrap();
let (parsed, _) = CompoundTag::from_binary_nbt(&binary, true).unwrap();
let parsed = CompoundTag::from_snbt(r#"{"key": "value"}"#).unwrap();

// 查询
tag.contains("Name");   // true
tag.empty();            // false
tag.get("RepairCost");  // Some(&Tag::Int(1))
tag.at("Name");         // &Tag::String(...), panic if missing

// 便捷方法
tag.rename("Name", "display_name");    // 重命名 key
tag.put("NewKey", "unique_value");     // 仅在 key 不存在时插入
tag.merge(&other, false);              // 合并另一个 CompoundTag

// 校验
CompoundTag::validate_network_nbt(&network);  // true
CompoundTag::validate_binary_nbt(&binary, true);  // true
```

## 支持的类型

| Tag 类型 | Rust 对应 | 自动转换来源 |
|---|---|---|
| `Tag::Byte` | `i8` | `bool`, `u8` |
| `Tag::Short` | `i16` | |
| `Tag::Int` | `i32` | `u16` |
| `Tag::Long` | `i64` | `u32` |
| `Tag::Float` | `f32` | |
| `Tag::Double` | `f64` | |
| `Tag::String` | `String` | `&str` |
| `Tag::ByteArray` | `Vec<u8>` | |
| `Tag::IntArray` | `Vec<i32>` | |
| `Tag::List` | `ListTagValue` | `Vec<String>`, `ListTag` |
| `Tag::Compound` | `HashMap<String, Tag>` | `CompoundTag` |

## NBT 格式

| 格式 | 方法 | 用途 |
|---|---|---|
| Little Endian | `to_binary_nbt(true, false)` / `from_binary_nbt(data, true)` | 标准 Bedrock NBT 文件 |
| Little Endian + Header | `to_binary_nbt(true, true)` / `from_binary_nbt_with_header(data, true)` | 带 `[version + size]` 头部的 LE |
| Big Endian | `to_binary_nbt(false, false)` / `from_binary_nbt(data, false)` | Java Edition 兼容 |
| Bedrock Network | `to_network_nbt()` / `from_network_nbt(data)` | Minecraft 网络传输（varint 前缀） |
| SNBT | `to_snbt()` / `from_snbt(snbt)` | 文本格式，用于调试 |

## 快速参考 API

### CompoundTag 方法

| 方法 | 返回 | 说明 |
|---|---|---|
| `new()` | `Self` | 创建空 compound |
| `set(key, value)` | - | 设置值（自动类型转换） |
| `get(key)` | `Option<&Tag>` | 取值 |
| `get_mut(key)` | `Option<&mut Tag>` | 可变取值 |
| `at(key)` | `&Tag` | 取值，key 不存在时 panic |
| `contains(key)` | `bool` | 检查 key 是否存在 |
| `remove(key)` | `bool` | 删除 key |
| `rename(old, new)` | `bool` | 重命名 key |
| `put(key, val)` | `bool` | key 不存在时插入 |
| `merge(other, merge_list)` | - | 合并另一个 compound |
| `size()` | `usize` | 条目数 |
| `empty()` | `bool` | 是否为空 |
| `keys()` | `impl Iterator<Item=&str>` | 遍历所有 key |
| `iter()` | `impl Iterator` | 遍历所有条目 |
| `iter_sorted()` | `Vec` | 按 key 排序遍历 |
| `to_network_nbt()` | `Vec<u8>` | 编码为 Network NBT |
| `to_binary_nbt(le, header)` | `Vec<u8>` | 编码为 LE/BE NBT |
| `to_binary_nbt_with_header(le, version)` | `Vec<u8>` | 编码为带 header 的 NBT |
| `to_snbt()` | `String` | 格式化为 SNBT 字符串 |
| `from_network_nbt(data)` | `Result` | 解析 Network NBT |
| `from_binary_nbt(data, le)` | `Result` | 解析 LE/BE NBT |
| `from_binary_nbt_with_header(data, le)` | `Result` | 解析带 header 的 NBT |
| `from_snbt(snbt)` | `Result` | 解析 SNBT 字符串 |
| `validate_network_nbt(data)` | `bool` | 校验 Network NBT |
| `validate_binary_nbt(data, le)` | `bool` | 校验 LE/BE NBT |

### ListTag 方法

| 方法 | 返回 | 说明 |
|---|---|---|
| `new()` | `Self` | 创建空列表 |
| `append(value)` | - | 追加元素（自动类型转换） |
| `get(index)` | `Option<&Tag>` | 按索引获取 |
| `size()` | `usize` | 元素个数 |
| `is_empty()` | `bool` | 是否为空 |
| `to_tag()` | `Tag` | 转换为 Tag::List |

## C FFI

通过 bedrock-ffi 暴露 18 个 C 函数以供调用。见 `crates/bedrock-ffi/README.md`。
