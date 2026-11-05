//! pygame_ozz —— 3D 骨骼动画适配层（阶段 4 绑定范式：rapier-starfish-debug
//! 兄弟 crate 同款——上游运行时 + 生态侧薄适配）。
//!
//! 上游 = [ozz-animation-rs](https://github.com/SlimeYummy/ozz-animation-rs)
//! （MPL-2.0，ozz-animation 的 Rust 移植——确定性采样 / 混合 / IK /
//! 蒙皮 job 管线）。**工具链前置：上游使用 std::simd——需 nightly rustc
//! 构建**（生态默认 stable；本 crate 经 pygamest feature `ozz` 门控，
//! nightly 宿主构建激活，stable 默认构建不受影响）。
//!
//! 本 crate 封装一个 [`Player`]（采样→局部→模型矩阵三段 job 的薄会话），
//! 并把 playback 测试资产内嵌为 demo 字节（零资产测试链；资产 MPL-2.0
//! 归属上游仓库）。
//!
//! 渲染接入路径（v1 数据面）：`model_matrices()` = 每关节 16 f32 列主
//! 矩阵流——引擎蒙皮 mesh draw 面落地后接 GPU；v1 为数据出口。

use glam::Mat4;
use ozz_animation_rs::{
    Animation, LocalToModelJob, SamplingContext, SamplingJob, Skeleton, SoaTransform,
};
use std::io::Cursor;
use std::sync::{Arc, RwLock};

/// demo 骨架（ozz playback 测试资产内嵌——MPL-2.0，归属上游仓库）
pub fn demo_skeleton_bytes() -> &'static [u8] {
    include_bytes!("../assets/skeleton.ozz")
}

/// demo 动画（同上）
pub fn demo_animation_bytes() -> &'static [u8] {
    include_bytes!("../assets/animation.ozz")
}

/// 采样→模型矩阵会话（job 泛型的薄会话化——Python 侧一个对象跑全程）。
pub struct Player {
    skeleton: Arc<Skeleton>,
    animation: Arc<Animation>,
    context: SamplingContext,
    locals: Arc<RwLock<Vec<SoaTransform>>>,
    models: Arc<RwLock<Vec<Mat4>>>,
}

impl Player {
    /// 从 ozz archive 字节构建（skeleton.ozz + animation.ozz）。
    pub fn new(
        skeleton_bytes: &[u8],
        animation_bytes: &[u8],
    ) -> Result<Self, ozz_animation_rs::OzzError> {
        let skeleton = {
            let mut ar = ozz_animation_rs::Archive::new(Cursor::new(skeleton_bytes))?;
            Skeleton::from_archive(&mut ar)?
        };
        let animation = {
            let mut ar = ozz_animation_rs::Archive::new(Cursor::new(animation_bytes))?;
            Animation::from_archive(&mut ar)?
        };
        let num_tracks = animation.num_tracks();
        Ok(Self {
            locals: Arc::new(RwLock::new(vec![
                SoaTransform::default();
                skeleton.num_soa_joints()
            ])),
            models: Arc::new(RwLock::new(vec![Mat4::IDENTITY; skeleton.num_joints()])),
            skeleton: Arc::new(skeleton),
            animation: Arc::new(animation),
            context: SamplingContext::new(num_tracks),
        })
    }

    /// 关节数。
    pub fn num_joints(&self) -> usize {
        self.skeleton.num_joints()
    }

    /// 关节名表（骨序——bimap 反查排序）。
    pub fn joint_names(&self) -> Vec<String> {
        let mut pairs: Vec<(i16, String)> = self
            .skeleton
            .joint_names()
            .iter()
            .map(|(name, idx)| (*idx, name.clone()))
            .collect();
        pairs.sort_by_key(|(idx, _)| *idx);
        pairs.into_iter().map(|(_, name)| name).collect()
    }

    /// 采样（ratio 0..1 归一动画时间）→ 模型矩阵（就地缓存）。
    pub fn sample(&mut self, ratio: f32) -> Result<(), ozz_animation_rs::OzzError> {
        let mut sampling = SamplingJob::default();
        sampling.set_animation(self.animation.clone());
        sampling.set_context(self.context.clone());
        sampling.set_ratio(ratio.clamp(0.0, 1.0));
        sampling.set_output(self.locals.clone());
        sampling.run()?;

        let mut l2m = LocalToModelJob::default();
        l2m.set_skeleton(self.skeleton.clone());
        l2m.set_input(self.locals.clone());
        l2m.set_output(self.models.clone());
        l2m.run()
    }

    /// 模型矩阵流（每关节 16 f32 列主——上一次 sample 的缓存）。
    pub fn model_matrices(&self) -> Vec<f32> {
        self.models
            .read()
            .map(|m| m.iter().flat_map(|x| x.to_cols_array()).collect())
            .unwrap_or_default()
    }

    /// 动画时长（秒）。
    pub fn duration(&self) -> f32 {
        self.animation.duration()
    }

    /// 关节世界位置（xy 平面——上一次 sample 的缓存；骨骼可视化 /
    /// 平面投影 demo 的消费形态）。
    pub fn joint_positions_xy(&self) -> Vec<(f32, f32)> {
        self.models
            .read()
            .map(|m| m.iter().map(|t| (t.w_axis.x, t.w_axis.y)).collect())
            .unwrap_or_default()
    }

    /// 父关节索引表（骨序；根 = -1）——骨骼连线段的拓扑。
    pub fn joint_parents(&self) -> Vec<i16> {
        self.skeleton.joint_parents().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_pipeline_samples() {
        let mut p = Player::new(demo_skeleton_bytes(), demo_animation_bytes())
            .expect("demo 资产加载");
        assert!(p.num_joints() > 0, "关节数 > 0");
        assert!(p.duration() > 0.0);
        p.sample(0.0).expect("采样 ratio=0");
        let m0 = p.model_matrices();
        assert_eq!(m0.len(), p.num_joints() * 16);
        assert!(m0.iter().all(|v| v.is_finite()), "矩阵全有限值");
        p.sample(0.5).expect("采样 ratio=0.5");
        assert_eq!(p.model_matrices().len(), m0.len());
    }

    #[test]
    fn joint_names_resolve() {
        let p = Player::new(demo_skeleton_bytes(), demo_animation_bytes()).unwrap();
        let names = p.joint_names();
        assert_eq!(names.len(), p.num_joints());
        assert!(names.iter().all(|n| !n.is_empty()));
    }
}
