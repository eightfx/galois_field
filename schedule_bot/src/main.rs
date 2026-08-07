//! LoL 日程調整 Discord bot。
//!
//! 毎週決まった曜日に候補日(「8月10日(月) 21:00から」形式)をチャンネルへ投稿し、
//! 各メッセージに ⭕ リアクションを付ける。メンバーの ⭕ が REQUIRED_COUNT 人
//! (デフォルト 5 人)に達した日を開催日として通知する。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Asia::Tokyo;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ChannelId, Client, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EventHandler,
    GatewayIntents, Http, Interaction, Reaction, ReactionType, Ready, UserId,
};
use serenity::async_trait;

const CIRCLE: &str = "⭕";
const WEEKDAY_JA: [&str; 7] = ["月", "火", "水", "木", "金", "土", "日"];

struct Config {
    token: String,
    channel_id: ChannelId,
    required_count: usize,
    /// 0=月曜 ... 6=日曜
    post_weekday: u32,
    post_hour: u32,
    days_ahead: u32,
    game_time: String,
    state_file: PathBuf,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
static BOT_ID: OnceLock<UserId> = OnceLock::new();
static LOOP_STARTED: AtomicBool = AtomicBool::new(false);

fn config() -> &'static Config {
    CONFIG.get().expect("config initialized in main")
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

impl Config {
    fn from_env() -> Self {
        Self {
            token: std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN を設定してください"),
            channel_id: ChannelId::new(
                std::env::var("CHANNEL_ID")
                    .expect("CHANNEL_ID を設定してください")
                    .parse()
                    .expect("CHANNEL_ID は数値で指定してください"),
            ),
            required_count: env_or("REQUIRED_COUNT", "5").parse().expect("REQUIRED_COUNT"),
            post_weekday: env_or("POST_WEEKDAY", "0").parse().expect("POST_WEEKDAY"),
            post_hour: env_or("POST_HOUR", "12").parse().expect("POST_HOUR"),
            days_ahead: env_or("DAYS_AHEAD", "7").parse().expect("DAYS_AHEAD"),
            game_time: env_or("GAME_TIME", "21:00"),
            state_file: env_or("STATE_FILE", "state.json").into(),
        }
    }
}

/// 候補日メッセージ ID → 日付と通知済みフラグ
#[derive(Serialize, Deserialize)]
struct Entry {
    date: NaiveDate,
    announced: bool,
}

type State = HashMap<u64, Entry>;

fn load_state() -> State {
    std::fs::read_to_string(&config().state_file)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &State) {
    let json = serde_json::to_string_pretty(state).expect("state serializes");
    if let Err(e) = std::fs::write(&config().state_file, json) {
        eprintln!("state.json の保存に失敗: {e}");
    }
}

fn now_jst() -> DateTime<Tz> {
    Utc::now().with_timezone(&Tokyo)
}

fn format_date(d: NaiveDate) -> String {
    let weekday = WEEKDAY_JA[d.weekday().num_days_from_monday() as usize];
    format!("{}月{}日({})", d.month(), d.day(), weekday)
}

fn circle() -> ReactionType {
    ReactionType::Unicode(CIRCLE.to_string())
}

async fn post_schedule(http: &Arc<Http>) -> serenity::Result<()> {
    let cfg = config();
    let today = now_jst().date_naive();

    let mut state = load_state();
    // 過ぎた候補日は掃除する
    state.retain(|_, entry| entry.date >= today);

    cfg.channel_id
        .say(
            http,
            format!(
                "@here 今週の LoL 日程調整です!\n参加できる日に {CIRCLE} を付けてください。{}人そろった日に開催します🎮",
                cfg.required_count
            ),
        )
        .await?;

    for i in 1..=cfg.days_ahead {
        let d = today + Duration::days(i.into());
        let msg = cfg
            .channel_id
            .say(http, format!("📅 {} {}から", format_date(d), cfg.game_time))
            .await?;
        msg.react(http, circle()).await?;
        state.insert(msg.id.get(), Entry { date: d, announced: false });
    }

    save_state(&state);
    println!("候補日を{}日分投稿しました({today}時点)", cfg.days_ahead);
    Ok(())
}

/// 次の「POST_WEEKDAY 曜日 POST_HOUR 時(JST)」を返す
fn next_post_time(now: DateTime<Tz>) -> DateTime<Tz> {
    let cfg = config();
    let today = now.date_naive();
    let days_until =
        (i64::from(cfg.post_weekday) - i64::from(today.weekday().num_days_from_monday())).rem_euclid(7);
    let at = |d: NaiveDate| {
        d.and_hms_opt(cfg.post_hour, 0, 0)
            .expect("valid hour")
            .and_local_timezone(Tokyo)
            .single()
            .expect("JST has no DST")
    };
    let candidate = at(today + Duration::days(days_until));
    if candidate > now {
        candidate
    } else {
        at(today + Duration::days(days_until + 7))
    }
}

async fn weekly_loop(http: Arc<Http>) {
    loop {
        let now = now_jst();
        let next = next_post_time(now);
        println!("次の定期投稿: {next}");
        if let Ok(wait) = (next - now).to_std() {
            tokio::time::sleep(wait).await;
        }
        if let Err(e) = post_schedule(&http).await {
            eprintln!("定期投稿に失敗: {e}");
        }
        // 同じ時刻に二重投稿しないよう1分待ってから次を計算する
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let _ = BOT_ID.set(ready.user.id);

        let commands = vec![
            CreateCommand::new("schedule").description("日程調整の候補日を今すぐ投稿します"),
            CreateCommand::new("next").description("次の開催日を表示します"),
        ];
        for guild in &ready.guilds {
            if let Err(e) = guild.id.set_commands(&ctx.http, commands.clone()).await {
                eprintln!("コマンド登録に失敗 (guild {}): {e}", guild.id);
            }
        }

        if !LOOP_STARTED.swap(true, Ordering::SeqCst) {
            tokio::spawn(weekly_loop(ctx.http.clone()));
        }
        println!("ログインしました: {}", ready.user.name);
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let cfg = config();
        if !reaction.emoji.unicode_eq(CIRCLE) {
            return;
        }
        if reaction.user_id == BOT_ID.get().copied() {
            return;
        }

        let mut state = load_state();
        let Some(entry) = state.get_mut(&reaction.message_id.get()) else {
            return;
        };
        if entry.announced {
            return;
        }
        let date = entry.date;

        let Ok(msg) = ctx.http.get_message(reaction.channel_id, reaction.message_id).await else {
            return;
        };
        let Ok(users) = msg.reaction_users(&ctx.http, circle(), Some(100), None).await else {
            return;
        };
        let participants: Vec<_> = users.iter().filter(|u| !u.bot).collect();
        if participants.len() < cfg.required_count {
            return;
        }

        entry.announced = true;
        save_state(&state);

        let mentions: Vec<String> = participants.iter().map(|u| format!("<@{}>", u.id)).collect();
        let text = format!(
            "🎉 {}人そろいました!\n🎮 次回は **{} {}から** です!\n参加者: {}",
            cfg.required_count,
            format_date(date),
            cfg.game_time,
            mentions.join(" ")
        );
        if let Err(e) = cfg.channel_id.say(&ctx.http, text).await {
            eprintln!("開催通知の送信に失敗: {e}");
        } else {
            println!("開催日が確定しました: {date}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(cmd) = interaction else {
            return;
        };
        match cmd.data.name.as_str() {
            "schedule" => {
                let defer = CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                );
                let _ = cmd.create_response(&ctx.http, defer).await;
                let content = match post_schedule(&ctx.http).await {
                    Ok(()) => "候補日を投稿しました!".to_string(),
                    Err(e) => format!("投稿に失敗しました: {e}"),
                };
                let followup = CreateInteractionResponseFollowup::new()
                    .content(content)
                    .ephemeral(true);
                let _ = cmd.create_followup(&ctx.http, followup).await;
            }
            "next" => {
                let today = now_jst().date_naive();
                let next = load_state()
                    .values()
                    .filter(|e| e.announced && e.date >= today)
                    .map(|e| e.date)
                    .min();
                let content = match next {
                    Some(d) => format!(
                        "🎮 次回は **{} {}から** です!",
                        format_date(d),
                        config().game_time
                    ),
                    None => format!(
                        "まだ開催日は決まっていません。候補日に {CIRCLE} を付けてください!"
                    ),
                };
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(content),
                );
                let _ = cmd.create_response(&ctx.http, response).await;
            }
            _ => {}
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = dotenvy::dotenv();
    let _ = CONFIG.set(Config::from_env());

    // キャッシュ機能は使わない(メモリ節約)。必要な intent は最小限。
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(&config().token, intents)
        .event_handler(Handler)
        .await
        .expect("クライアントの作成に失敗");

    if let Err(e) = client.start().await {
        eprintln!("bot の起動に失敗: {e}");
    }
}
