use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use ffxiv_crafting::{export, Attributes, Recipe, Skills, Status};
use std::cmp::Ordering;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let theme = ColorfulTheme::default();
    println!("欢迎使用FFXIV生产宏深度优先搜索程序");
    loop {
        let attributes = Attributes {
            level: Input::with_theme(&theme)
                .with_prompt("玩家等级(player level)")
                .default(80)
                .interact()?,
            craftsmanship: Input::with_theme(&theme)
                .with_prompt("制作精度(craftsmanship)")
                .interact()?,
            control: Input::with_theme(&theme)
                .with_prompt("加工精度(control)")
                .interact()?,
            craft_points: Input::with_theme(&theme)
                .with_prompt("制作力(craft points)")
                .interact()?,
        };
        let rlv = Input::with_theme(&theme)
            .with_prompt("配方品级(recipe level)")
            .interact()?;
        let recipe = Recipe::new(
            rlv,
            Input::with_theme(&theme)
                .with_prompt("配方玩家等级(recipe player level)")
                .default(rlv_to_job_level(rlv))
                .interact()?,
            Input::with_theme(&theme)
                .with_prompt("配方难度(difficulty)")
                .interact()?,
            Input::with_theme(&theme)
                .with_prompt("配方品质(quality)")
                .interact()?,
            Input::with_theme(&theme)
                .with_prompt("配方耐久(durability)")
                .interact()?,
            Input::with_theme(&theme)
                .with_prompt("制作状态标志(conditions flag)")
                .default(15)
                .interact()?,
        );
        let status = Status::new(attributes, recipe);
        let depth = Input::with_theme(&theme)
            .with_prompt("限制搜索深度(search depth)")
            .default(6)
            .interact()?;
        println!("正在进行深度优先搜索，请稍等");
        let (best_list, Score { quality, steps }) = dfs_search(&status, depth);
        println!(
            "{}# 品质：{}，步数：{}",
            export::to_chinese_macro(&best_list),
            quality,
            steps,
        );
        if 0 == Select::with_theme(&theme)
            .with_prompt("搜索完毕，是否再来一次？")
            .default(0)
            .item("退出")
            .item("重试")
            .interact()? {
            return Ok(());
        }
    }
}

/// 进行一次深度优先搜索（DFS）
///
/// status为开始制作时的初始状态
/// maximum_depth为限制最深搜索深度
fn dfs_search(status: &Status, maximum_depth: i32) -> (Vec<Skills>, Score) {
    let mut best_list = Vec::new();
    let mut best_score = Score::from(status);
    search(
        &status,
        0,
        maximum_depth,
        &mut Vec::with_capacity(maximum_depth as usize),
        &mut best_list,
        &mut best_score,
    );
    fn search(
        status: &Status,
        current_depth: i32,
        maximum_depth: i32,
        stack_seq: &mut Vec<Skills>,
        best_seq: &mut Vec<Skills>,
        best_score: &mut Score,
    ) {
        let score = Score::from(status);
        if score > *best_score {
            *best_score = score;
            *best_seq = stack_seq.clone();
        }
        // 简单的剪枝，排除比当前最优解更深的分支。
        let is_best_quality_full = best_score.quality >= status.recipe.quality;
        let is_this_steps_longer = current_depth >= best_score.steps;
        if is_best_quality_full && is_this_steps_longer || current_depth > maximum_depth {
            return;
        }
        for sk in SKILL_LIST {
            let mut new_s = *status;
            if new_s.is_action_allowed(sk).is_ok() {
                new_s.cast_action(sk);
                stack_seq.push(sk);
                search(
                    &new_s,
                    current_depth + 1,
                    maximum_depth,
                    stack_seq,
                    best_seq,
                    best_score,
                );
                stack_seq.pop();
            }
        }
    }
    (best_list, best_score)
}

#[derive(Copy, Clone, Eq)]
struct Score {
    quality: i32,
    steps: i32,
}

impl From<&Status> for Score {
    fn from(s: &Status) -> Self {
        Self {
            quality: if s.progress >= s.recipe.progress {
                s.quality.min(s.recipe.quality)
            } else {
                0
            },
            steps: s.step,
        }
    }
}

impl Ord for Score {
    /// 定义Score之间的偏序关系，这里不使用Ord是因为避免麻烦
    /// * 品质更高的分数更高
    /// * 品质相同，步数越短分数越高
    /// * 品质和步数都相同，两个分数相等
    fn cmp(&self, other: &Self) -> Ordering {
        match self.quality.cmp(&other.quality) {
            Ordering::Equal => self.steps.cmp(&other.steps).reverse(),
            x => x,
        }
    }
}

impl PartialOrd<Self> for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Self> for Score {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

/// 搜索的技能列表
#[allow(dead_code)]
const SKILL_LIST: [Skills; 24] = [
    Skills::MuscleMemory,
    Skills::Reflect,
    // Skills::TrainedEye,
    Skills::BasicSynthesis,
    // Skills::RapidSynthesis,
    Skills::BrandOfTheElements,
    Skills::CarefulSynthesis,
    // Skills::FocusedSynthesis,
    Skills::Groundwork,
    // Skills::IntensiveSynthesis,
    Skills::DelicateSynthesis,
    Skills::BasicTouch,
    // Skills::HastyTouch,
    Skills::StandardTouch,
    Skills::ByregotsBlessing,
    // Skills::PreciseTouch,
    // Skills::PatientTouch,
    Skills::PrudentTouch,
    // Skills::FocusedTouch,
    Skills::PreparatoryTouch,
    Skills::TricksOfTheTrade,
    Skills::MastersMend,
    Skills::WasteNot,
    Skills::WasteNotII,
    Skills::Manipulation,
    Skills::InnerQuiet,
    Skills::Veneration,
    Skills::GreatStrides,
    Skills::Innovation,
    Skills::NameOfTheElements,
    Skills::Observe,
    Skills::FinalAppraisal,
];

fn rlv_to_job_level(rlv: i32) -> i32 {
    match rlv {
        x if x < 50 => x,
        x if x < 115 => 50,
        x if x < 124 => 51,
        x if x < 130 => 52,
        x if x < 133 => 53,
        x if x < 136 => 54,
        x if x < 139 => 55,
        x if x < 142 => 56,
        x if x < 145 => 57,
        x if x < 148 => 58,
        x if x < 150 => 59,
        x if x < 255 => 60,
        x if x < 265 => 61,
        x if x < 270 => 62,
        x if x < 273 => 63,
        x if x < 276 => 64,
        x if x < 279 => 65,
        x if x < 282 => 66,
        x if x < 285 => 67,
        x if x < 288 => 68,
        288 | 289 => 69,
        x if x < 381 => 70,
        x if x < 395 => 71,
        x if x < 400 => 72,
        x if x < 403 => 73,
        x if x < 406 => 74,
        x if x < 409 => 75,
        x if x < 412 => 76,
        x if x < 415 => 77,
        x if x < 418 => 78,
        x if x < 430 => 79,
        _ => 80,
    }
}
