# LoL 日程調整 bot

5 人サーバー向けの Discord 日程調整 bot。

## 動き

1. 毎週決まった曜日(デフォルト: 月曜 12:00 JST)に、翌日から 7 日分の候補日を
   `📅 8月10日(月) 21:00から` の形式でチャンネルに投稿する
2. bot が各候補日に ⭕ リアクションを付けておく
3. 参加できる日にメンバーが ⭕ を押す
4. bot 以外の ⭕ が 5 人に達した瞬間、
   `🎮 次回は 8月10日(月) 21:00から です!` と参加者メンション付きで通知する

スラッシュコマンド:

| コマンド | 説明 |
| --- | --- |
| `/schedule` | 候補日をその場で投稿(初回やテストに) |
| `/next` | 次の確定した開催日を表示 |

## セットアップ

### 1. bot を作る

1. [Discord Developer Portal](https://discord.com/developers/applications) → **New Application**
2. **Bot** タブ → **Reset Token** でトークンをコピー(privileged intents は不要)
3. **OAuth2 → URL Generator** で scope に `bot` と `applications.commands`、
   permission に **View Channels / Send Messages / Add Reactions / Read Message History /
   Mention Everyone** を選び、生成された URL からサーバーに招待する

### 2. 動かす

```bash
cd schedule_bot
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

cp .env.example .env
# .env に DISCORD_TOKEN とチャンネル ID を書く

python bot.py
```

起動後に `/schedule` を打てばすぐ候補日が投稿されるので、動作確認はそれが早いです。

## 設定(環境変数)

| 変数 | デフォルト | 説明 |
| --- | --- | --- |
| `DISCORD_TOKEN` | (必須) | bot トークン |
| `CHANNEL_ID` | (必須) | 候補日を投稿するチャンネル ID |
| `REQUIRED_COUNT` | `5` | 開催に必要な ⭕ の人数 |
| `POST_WEEKDAY` | `0` | 投稿する曜日(0=月 〜 6=日) |
| `POST_HOUR` | `12` | 投稿する時刻(JST) |
| `DAYS_AHEAD` | `7` | 何日先まで候補日を出すか |
| `GAME_TIME` | `21:00` | 開催時刻の表示 |
| `STATE_FILE` | `state.json` | 候補日メッセージの記録先 |

## メモ

- 候補日メッセージと確定状況は `state.json` に保存されるので、bot を再起動しても
  過去に投稿した候補日への ⭕ は引き続き検知される
- 一度確定を通知した日は、その後 ⭕ が外されても取り消しはしない(手動で相談してください)
- 常時起動が必要なので、Raspberry Pi・VPS・無料枠のホスティングなどで
  `python bot.py` を動かしっぱなしにするのがおすすめ
  (systemd を使うなら `Restart=always` を付けると安心)
