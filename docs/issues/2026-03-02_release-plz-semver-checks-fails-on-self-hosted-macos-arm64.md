# Issue #60: release-plz `release-pr` fails on self-hosted macOS ARM64

## Context

Release PR のワークフローで `release-plz/action` 実行中に `cargo-semver-checks` バイナリ取得が失敗し、`Release PR` ジョブがエラー終了する。

主なログ:

- `System reported platform: darwin`
- `System reported arch: arm64`
- `Error: Could not find a release for v0.30.0. Found: cargo-semver-checks-aarch64-apple-darwin.tar.gz, ...`

## Root Cause

self-hosted runner（`macOS` / `ARM64`）上で、`release-plz` が内部的に利用する `cargo-semver-checks` のアセット解決が失敗している。

- runner 判定: `darwin` + `arm64`
- 実アセット命名: `aarch64-apple-darwin`
- インストーラ側の解決ロジックと命名規則が噛み合わず、取得に失敗

`Cache service responded with 400` は副次的な警告であり、主因ではない。

## Fix

`release-plz.toml` で semver check を無効化する。

```toml
[workspace]
semver_check = false
```

このプロジェクトは現状バイナリ中心で、`cargo-semver-checks` 依存を外してもリリース運用に影響しない。

## Validation

1. `release-plz` の `Release PR` ジョブを再実行する
2. `cargo-semver-checks` の取得ステップが呼ばれないことを確認する
3. Release PR が正常作成されることを確認する
4. マージ後にタグ/Release が作成されることを確認する
