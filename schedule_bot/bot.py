"""LoL 日程調整 Discord bot.

毎週決まった曜日に候補日(「8月10日(月) 21:00から」形式)をチャンネルへ投稿し、
各メッセージに ⭕ リアクションを付ける。メンバーの ⭕ が REQUIRED_COUNT 人
(デフォルト 5 人)に達した日を開催日として通知する。
"""

import datetime
import json
import logging
import os
from pathlib import Path
from zoneinfo import ZoneInfo

import discord
from discord import app_commands
from discord.ext import tasks

try:  # .env があれば読む(無くても動く)
    from dotenv import load_dotenv

    load_dotenv()
except ImportError:
    pass

JST = ZoneInfo("Asia/Tokyo")
CIRCLE = "⭕"
WEEKDAY_JA = ["月", "火", "水", "木", "金", "土", "日"]

TOKEN = os.environ["DISCORD_TOKEN"]
CHANNEL_ID = int(os.environ["CHANNEL_ID"])
REQUIRED_COUNT = int(os.environ.get("REQUIRED_COUNT", "5"))
POST_WEEKDAY = int(os.environ.get("POST_WEEKDAY", "0"))  # 0=月曜 ... 6=日曜
POST_HOUR = int(os.environ.get("POST_HOUR", "12"))
DAYS_AHEAD = int(os.environ.get("DAYS_AHEAD", "7"))
GAME_TIME = os.environ.get("GAME_TIME", "21:00")
STATE_FILE = Path(os.environ.get("STATE_FILE", "state.json"))

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger("schedule_bot")


def load_state() -> dict:
    if STATE_FILE.exists():
        return json.loads(STATE_FILE.read_text(encoding="utf-8"))
    return {}


def save_state(state: dict) -> None:
    STATE_FILE.write_text(
        json.dumps(state, ensure_ascii=False, indent=2), encoding="utf-8"
    )


def format_date(d: datetime.date) -> str:
    return f"{d.month}月{d.day}日({WEEKDAY_JA[d.weekday()]})"


intents = discord.Intents.default()
client = discord.Client(intents=intents)
tree = app_commands.CommandTree(client)


async def get_channel() -> discord.TextChannel:
    channel = client.get_channel(CHANNEL_ID)
    if channel is None:
        channel = await client.fetch_channel(CHANNEL_ID)
    return channel


async def post_schedule(channel: discord.TextChannel) -> None:
    today = datetime.datetime.now(JST).date()
    state = load_state()

    # 過ぎた候補日は state から掃除する
    state = {
        mid: entry
        for mid, entry in state.items()
        if datetime.date.fromisoformat(entry["date"]) >= today
    }

    await channel.send(
        f"@here 今週の LoL 日程調整です!\n"
        f"参加できる日に {CIRCLE} を付けてください。"
        f"{REQUIRED_COUNT}人そろった日に開催します🎮"
    )
    for i in range(1, DAYS_AHEAD + 1):
        d = today + datetime.timedelta(days=i)
        msg = await channel.send(f"📅 {format_date(d)} {GAME_TIME}から")
        await msg.add_reaction(CIRCLE)
        state[str(msg.id)] = {"date": d.isoformat(), "announced": False}

    save_state(state)
    log.info("candidate dates posted: %s days from %s", DAYS_AHEAD, today)


@client.event
async def on_raw_reaction_add(payload: discord.RawReactionActionEvent) -> None:
    if payload.user_id == client.user.id:
        return
    if str(payload.emoji) != CIRCLE:
        return

    state = load_state()
    entry = state.get(str(payload.message_id))
    if entry is None or entry["announced"]:
        return

    channel = await get_channel()
    message = await channel.fetch_message(payload.message_id)
    reaction = next(
        (r for r in message.reactions if str(r.emoji) == CIRCLE), None
    )
    if reaction is None:
        return

    users = [u async for u in reaction.users() if not u.bot]
    if len(users) < REQUIRED_COUNT:
        return

    entry["announced"] = True
    save_state(state)

    d = datetime.date.fromisoformat(entry["date"])
    mentions = " ".join(u.mention for u in users)
    await channel.send(
        f"🎉 {REQUIRED_COUNT}人そろいました!\n"
        f"🎮 次回は **{format_date(d)} {GAME_TIME}から** です!\n"
        f"参加者: {mentions}"
    )
    log.info("date confirmed: %s", d)


@tasks.loop(time=datetime.time(hour=POST_HOUR, minute=0, tzinfo=JST))
async def weekly_post() -> None:
    if datetime.datetime.now(JST).weekday() != POST_WEEKDAY:
        return
    await post_schedule(await get_channel())


@weekly_post.before_loop
async def before_weekly_post() -> None:
    await client.wait_until_ready()


@tree.command(name="schedule", description="日程調整の候補日を今すぐ投稿します")
async def schedule_command(interaction: discord.Interaction) -> None:
    await interaction.response.defer(ephemeral=True)
    await post_schedule(await get_channel())
    await interaction.followup.send("候補日を投稿しました!", ephemeral=True)


@tree.command(name="next", description="次の開催日を表示します")
async def next_command(interaction: discord.Interaction) -> None:
    today = datetime.datetime.now(JST).date()
    confirmed = sorted(
        datetime.date.fromisoformat(entry["date"])
        for entry in load_state().values()
        if entry["announced"]
        and datetime.date.fromisoformat(entry["date"]) >= today
    )
    if confirmed:
        await interaction.response.send_message(
            f"🎮 次回は **{format_date(confirmed[0])} {GAME_TIME}から** です!"
        )
    else:
        await interaction.response.send_message(
            f"まだ開催日は決まっていません。候補日に {CIRCLE} を付けてください!"
        )


@client.event
async def on_ready() -> None:
    await tree.sync()
    if not weekly_post.is_running():
        weekly_post.start()
    log.info("logged in as %s", client.user)


client.run(TOKEN)
