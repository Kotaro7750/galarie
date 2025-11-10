# Feature Specification: Galarie Media Platform Core

**Feature Branch**: `[001-galarie-media-platform]`  
**Created**: 2025-11-09  
**Status**: Draft  
**Input**: Local multimedia library with tag-encoded filenames  
**Constitution Version**: 1.0.0 (reference when completing Constitution Check)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Tag-Based Thumbnail Search (Priority: P1)

ローカルファイルに付与したタグ（`tag` と `key:value`）を使って AND 条件検索し、数千件規模でも 1 秒以内に結果サムネイルを確認できるようにする。

**Why this priority**: タグ検索こそが全機能の入口であり、ここが高速・直感的でなければ後続の体験（お気に入り、視聴）が成立しないため。

**Independent Test**: 既存ファイル群をインデックスし、`tagA AND tagB AND key1=valueX` のような検索で 1 秒以内にサムネイルグリッドが表示されることを確認する自動テストを実行する。

**Rollback Plan**: タグキャッシュやサムネイルキャッシュを削除し、API を停止してもファイル自体は変化しないため運用影響を最小化できる。

**Acceptance Scenarios**:

1. **Given** キャッシュが最新の状態, **When** ユーザーが複数タグ条件を入力, **Then** 1 秒以内にマッチしたファイルのサムネイルがグリッド表示され種類が区別できる。
2. **Given** key:value タグが付与されたファイル, **When** ユーザーがキーと値を複数指定, **Then** 完全一致したファイルのみが検索結果に含まれる。
3. **Given** ブラウザセッション中, **When** ユーザーが検索条件を変更, **Then** 変更内容が即座に反映され検索状態はセッション終了まで保持される。

---

### User Story 2 - Favorites Slideshow for Images/GIFs (Priority: P2)

検索結果から画像/GIF をお気に入りに登録し、指定した間隔で固定順・無限ループのスライドショーを全画面＆ピンチズーム対応で楽しむ。

**Why this priority**: タグ検索後の鑑賞体験が最大の価値であり、ユーザーは自分好みのコレクションを即座に再生できる必要があるため。

**Independent Test**: テストデータでお気に入りを選択し、スライドショーを開始→停止→再開してもお気に入りが維持され、設定した間隔でループすることを UI テストで確認する。

**Rollback Plan**: スライドショー関連のキャッシュと UI を無効化し、タグ検索のみ提供する状態に戻す。キャッシュ削除で影響範囲を限定。

**Acceptance Scenarios**:

1. **Given** ユーザーが検索結果グリッドを表示, **When** 任意のサムネイルをお気に入りに追加, **Then** お気に入りリストに順番付きで表示されブラウザを閉じるまで保持される。
2. **Given** お気に入りリストが存在, **When** ユーザーがスライドショーを開始し間隔を 3 秒に設定, **Then** 画像/GIF が固定順・無限ループで再生される。
3. **Given** スライドショー中, **When** ユーザーが全画面やピンチズームを使用, **Then** 表示が途切れずに拡大・縮小できる。

---

### User Story 3 - Video Loop & A-B Repeat (Priority: P3)

動画ファイルを同一タブ内で再生し、ループや A-B 区間リピートを操作しても検索・お気に入り状態を失わない視聴体験を提供する。

**Why this priority**: 画像/GIF 以外のコレクションにも応用し、視聴の柔軟性を高めることでマルチメディア体験を完成させるため。

**Independent Test**: テスト用動画で A 点/B 点を設定し、区間ループ→解除→全体ループを UI テストで検証する。

**Rollback Plan**: 動画プレイヤーのループ/A-B 機能を無効化し、標準再生のみに戻す。キャッシュを削除すれば副作用なし。

**Acceptance Scenarios**:

1. **Given** ユーザーが検索結果から動画を選択, **When** 同一タブ内で再生を開始, **Then** 全画面表示が可能になり検索条件やお気に入りは保持される。
2. **Given** 動画が再生中, **When** ユーザーが A 点/B 点を設定しリピートを有効化, **Then** 指定区間のみが継続的に再生される。
3. **Given** 区間リピートが有効, **When** ユーザーがループ設定を解除, **Then** 通常再生に戻り任意のタイミングで全体ループへ切り替えられる。

---

### Edge Cases

- 解析対象ディレクトリにアクセス権がない場合のエラー処理。
- タグに使用できない文字（ファイルシステム制約）を含むファイル名を検出した場合のスキップ/通知。
- 巨大な GIF や動画でサムネイル生成に失敗した際のフォールバック表示。
- ブラウザストレージが埋まった場合の状態保持失敗。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse tag metadata from filenames (`tag`, `key:value`) and cache the results for ≤1s query latency.
- **FR-002**: System MUST expose a RESTful `GET /api/v1/media` endpoint that executes AND-based tag queries with multi-value filters for key tags.
- **FR-003**: Users MUST be able to view search results as thumbnails with media-type indicators provided through the search response.
- **FR-004**: Frontend MUST maintain favorites and slideshow queues client-side (e.g., browser storage) so that state persists until the tab closes without backend storage.
- **FR-005**: System MUST provide a slideshow player for images/GIFs with configurable interval, fixed order, and infinite loop (client-controlled).
- **FR-006**: System MUST offer a video player with loop toggle and A-B repeat controls implemented entirely in the frontend, while the backend serves media streams.
- **FR-007**: System MUST export OpenTelemetry traces, metrics, and logs for indexing, search, slideshow, and video playback flows.
- **FR-008**: System MUST support full-screen and pinch-to-zoom gestures during viewing.

### Key Entities

- **MediaFile**: Represents a file path, media type (image/GIF/video), parsed tags, thumbnail location/URL.
- **TagFilter**: Collection of required tags and key-value filters used to query MediaFiles.
- **FavoriteSet**: Ordered list of MediaFiles selected during the session, including slideshow interval settings.

## Contract Surfaces *(mandatory when exposing an interface)*

- **Interface Name**: `GET /api/v1/media`
  - **Purpose**: Search media with AND-based tag filters.
  - **Request Schema**: Query parameters `tags=tag1,tag2`, `attributes[key]=value1,value2`, `page`, `pageSize`.
  - **Response Schema**: `{ items: MediaFile[], total: number, page: number, pageSize: number }` with REST error envelope `{ error: { code, message } }`.
  - **Versioning**: URI namespace `/api/v1`; breaking changes require `/api/v2`.
  - **Quickstart Step**: Quickstart “Search via REST API”.
  - **Required Tests**: Backend integration tests (e.g., Rust `tests/contract/media_search.rs` or Go equivalent) verifying AND semantics and pagination.

- **Interface Name**: `GET /api/v1/media/{id}/thumbnail`
  - **Purpose**: Retrieve thumbnail binary/URL for a media item (supports HTTP caching).
  - **Request Schema**: Path parameter `id`, optional `size`.
  - **Response Schema**: Binary image (Content-Type `image/jpeg`/`image/png`); 404 if missing.
  - **Versioning**: `/api/v1`.
  - **Quickstart Step**: “Fetch thumbnails” example.
  - **Required Tests**: Backend integration tests ensuring correct content type, caching headers, and 404 behavior.

- **Interface Name**: `GET /api/v1/media/{id}/stream`
  - **Purpose**: Stream the original media file for viewing/playing; supports Range requests.
  - **Request Schema**: Path parameter `id`, optional query `disposition=inline`.
  - **Response Schema**: Binary data with appropriate `Content-Type`; supports 206 Partial Content.
  - **Versioning**: `/api/v1`.
  - **Quickstart Step**: “Stream media file” example.
  - **Required Tests**: Backend integration tests validating Range support, MIME negotiation, and error handling.

- **Interface Name**: `POST /api/v1/index/rebuild`
  - **Purpose**: Trigger tag re-indexing (admin operation).
  - **Request Schema**: `{ force?: boolean }`.
  - **Response Schema**: `{ status: "queued" | "complete" }`.
  - **Versioning**: `/api/v1`.
  - **Quickstart Step**: “Rebuild index” example.
  - **Required Tests**: Backend integration tests verifying concurrency control and success/failure signaling.

## Success Criteria *(mandatory)*

- **SC-001**: Tag search API returns results for 2,000 files in ≤1s on target hardware.
- **SC-002**: Users can create a slideshow-ready favorite list in ≤3 interactions after search.
- **SC-003**: Video A-B repeat maintains playback without noticeable stalls (<100ms gap).
- **SC-004**: 95% of manual browsing sessions complete without cache regeneration errors.
- **SC-005**: OpenTelemetry pipeline exports ≥99% of spans/logs/metrics for indexing, search, slideshow, and video flows; profiling capture is optional but documented.
