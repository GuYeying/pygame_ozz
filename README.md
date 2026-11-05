# pygame_ozz

pygamest 生态 3D 骨骼动画适配层——[ozz-animation-rs](https://github.com/SlimeYummy/ozz-animation-rs)（MPL-2.0）的薄封装（`Player` 会话：采样→局部→模型矩阵三段 job）。

## ⚠️ 工具链前置（休眠状态）

上游使用 `std::simd`（portable_simd）——**要求 nightly rustc**。pygamest
生态主体运行在 stable（1.97 stable-msvc），故本 crate 当前为**休眠交付**：
代码与 demo 资产就绪、`cargo test` 需 nightly 环境（`rustup toolchain
install nightly` 后在nightly 下验证）。生态若迁移 nightly 即可激活接入
pygame-rs（feature `ozz` + rpy 绑定，效仿 pygame_spine 范式）。

## 用法（nightly 下）

```rust
let mut p = Player::new(demo_skeleton_bytes(), demo_animation_bytes())?;
p.sample(0.5)?;                       // ratio 0..1
let mats = p.model_matrices();        // 每关节 16 f32 列主
```
