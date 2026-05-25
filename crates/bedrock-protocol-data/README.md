# bedrock-protocol-data

预生成的 Bedrock 协议版本数据。JSON blob 在构建时嵌入二进制。

## 用法

```rust
use bedrock_protocol_data::registry;

// 一次调用获取所有嵌入版本的注册表
let reg = registry();
let v975 = reg.get(975).unwrap();
let latest = reg.latest();
```

## 添加新协议版本

### Step 1：生成数据文件

```bash
# 从 protocol-docs 的 Git 分支生成
cargo run -p bedrock-protocol-data --bin generate-data -- \
  --docs /path/to/protocol-docs \
  --output ./crates/bedrock-protocol-data/data/v1001.json
```

参数说明：
- `--docs`：指向 protocol-docs 目录（含 packets/、enums/、types/ 子目录）
- `--output`：输出路径，惯例放在 `data/v{协议号}.json`

### Step 2：更新 src/lib.rs

打开 `crates/bedrock-protocol-data/src/lib.rs`，在 `blobs` 数组中加入新版本：

```rust
pub fn registry() -> ProtocolRegistry {
    // ── Add new versions below ──────────────────────────────────────────
    let blobs: &[&str] = &[
        include_str!("../data/v897.json"),   // r21_u13  (1.21.130.28)
        include_str!("../data/v924.json"),   // r26_u0   (1.26.0.29)
        include_str!("../data/v944.json"),   // r26_u1   (1.26.10.27)
        include_str!("../data/v975.json"),   // r26_u2   (1.26.20.28) ← current
        include_str!("../data/v1001.json"),  // r26_u3   (1.26.30.30)
        include_str!("../data/v{N}.json"),  // ← 新增这行
    ];
    // ────────────────────────────────────────────────────────────────────

    ProtocolRegistry::from_embedded(blobs)
        .expect("Failed to build protocol registry from embedded data")
}
```

### Step 3：提交

```bash
git add crates/bedrock-protocol-data/data/v1001.json
git commit -m "add protocol v1001 data"
```

## 文件结构

```
bedrock-protocol-data/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # registry() — 加载所有嵌入版本
│   │                            #   ↑ 添加新版本时修改此文件
│   └── bin/
│       └── generate-data.rs    # 命令行生成工具
│                               #   cargo run --bin generate-data -- --docs ... --output ...
└── data/
    ├── v897.json               # r21_u13  (1.21.130.28)
    ├── v924.json               # r26_u0   (1.26.0.29)
    ├── v944.json               # r26_u1   (1.26.10.27)
    ├── v975.json               # r26_u2   (1.26.20.28) ← current
    └── v1001.json              # r26_u3   (1.26.30.30)
```

## 验证

```bash
# 查看已嵌入的数据统计
python3 -c "
import json
with open('crates/bedrock-protocol-data/data/v975.json') as f:
    d = json.load(f)
print(f'Version {d[\"network_version\"]} ({d[\"branch_name\"]}), MC {d[\"minecraft_version\"]}')
print(f'  {len(d[\"packets\"])} packets, {len(d[\"enums\"])} enums, {len(d[\"types\"])} types')
"

# 或者直接运行测试
cargo test -p bedrock-protocol-data
```

## 设计原则

- **数据即代码**：每次协议更新跑一次 `generate-data`，产物提交 git
- **多版本共存**：`registry()` 同时返回所有嵌入版本，按协议号索引
