# webdemo

```bash
# 监听
cargo watch -w webapp/ -x run

# 需要交叉平台时建议使用 WSL 或 虚拟机 Docker 替代。
cargo build --release 

# 交叉编译 linux(配置麻烦)
cargo build --release --target=x86_64-unknown-linux-gnu
```