# PHP
- http
    - GET /
        - 首頁
        - 輸入key進入房間 (新用戶) -> /room/$key/$id
    - GET /room/$key/$id
        - 房間
        - 如果沒有這個id 或ip不符合, 顯示簡單的錯誤畫面, 引導建立新的user -> /api/newuser?key=$key

# Rust
- http
    - GET /api/newuser?key=$roomkey -> /room/$key/$id
- ws
    - /api/room?key=$roomkey&id=$userid -> message stream
