# pygame_ozz

pygamest 生态 3D 骨骼动画适配层——[ozz-animation-rs](https://github.com/SlimeYummy/ozz-animation-rs)（MPL-2.0）的薄封装（`Player` 会话：采样→局部→模型矩阵三段 job）。

## 工具链前置（nightly 专属）

上游使用 `std::simd`（portable_simd）——**要求 nightly rustc**。生态
默认 stable 不受影响：pygamest 侧 feature `ozz` 门控（不入 default），
nightly 宿主构建激活（`cargo +nightly build --features ozz`）。

**激活状态（2026-11-06）**：nightly 已装（默认通道仍 stable）、Player
采样链 `cargo +nightly test` 2/2 绿、rpy 绑定已接入 pygamest（feature
`ozz`）、桌面 Python smoke 通过。

## 用法

```rust
let mut p = Player::new(demo_skeleton_bytes(), demo_animation_bytes())?;
p.sample(0.5)?;                       // ratio 0..1
let mats = p.model_matrices();        // 每关节 16 f32 列主
```

Python 面（nightly 宿主）：

```python
import pygame_ozz
p = pygame_ozz.demo_player()
p.sample(0.5)
mats = p.model_matrices()             # 每关节 16 f32 列主
```
