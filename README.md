# Webtale-Engine

Webブラウザを使用して自由にカスタマイズ可能なUndertale風の戦闘を楽しめるソフトウェアです。本プロジェクトはTobyFox氏および本家Undertaleとは一切の関係がありません。

## フェーズスクリプト

`projects/<project>/phases` に `.wep` を配置し、`enemyStatus.wep` の `phaseScript` で初期フェーズ名(拡張子なし)を指定します。`update(context)` は `trigger` が `start` / `turn` / `damage` のタイミングで呼ばれます。

`phase_api.wep` の関数でダイアログや攻撃パターンなどを更新できます。

- `setDialogText(text)`
- `setAttackPatterns(patterns)`
- `setBubbleMessages(messages)`
- `setBubbleMessage(message)`
- `setBubbleTexture(path)`
- `setNextPhase(name)`

`context` には `turn`, `phaseTurn`, `enemyHp`, `enemyMaxHp`, `enemyName`, `phase`, `isFirstTurn`, `isPhaseStart`, `lastPlayerAction`, `lastActCommand` が入ります。
`lastPlayerAction` は `attackHit`, `attackMiss`, `act`, `item`, `spare`, `flee` が入ります。
