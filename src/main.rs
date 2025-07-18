use chrono::{DateTime, Local};
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
// #[derive(Debug)]
struct AccountData {
    // timestamp: DateTime<Local>,
    followers: u64,
    following: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = "your_target_instagram_username"; // 替換成你要追蹤的公開帳號
    let url = format!("https://www.instagram.com/{}/", username);

    // 1. 發送HTTP請求
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    let response = client.get(&url).send().await?.text().await?;

    // 2. 解析HTML (這裡簡化，假設能直接從HTML找到)
    // Instagram 的頁面結構複雜且動態，直接解析 HTML 數字會非常脆弱
    // 更穩健的做法是尋找 <script> 標籤中包含的 JSON-LD 或其他數據
    let document = Html::parse_document(&response);

    // 這些選擇器需要你手動檢查 Instagram 頁面的 HTML 結構來確定
    // 舉例：Instagram 數量可能在 <meta property="og:description" content="... followers, ... following"> 中
    // 或者在某個 JavaScript 變數裡
    // 這裡我們假設它們存在於某個 `<span>` 標籤中，這很可能是不準確的
    let followers_selector = Selector::parse("span.g47sy").unwrap(); // 假設這是追蹤者數量的選擇器
    let following_selector = Selector::parse("span.g47sy").unwrap(); // 假設這是追蹤對象數量的選擇器

    let mut current_followers: Option<u64> = None;
    let mut current_following: Option<u64> = None;

    // 嘗試從 HTML 裡提取數字 (這部分非常脆弱，容易失效)
    for element in document.select(&followers_selector) {
        if let Some(text) = element.text().next() {
            // 需要進一步處理字符串，例如移除逗號，然後解析成數字
            // 例如: "1,234" -> 1234
            if let Ok(num) = text.replace(",", "").parse::<u64>() {
                current_followers = Some(num);
                break;
            }
        }
    }

    // 同樣處理 following 數量
    for element in document.select(&following_selector) {
        if let Some(text) = element.text().next() {
            if let Ok(num) = text.replace(",", "").parse::<u64>() {
                current_following = Some(num);
                break;
            }
        }
    }

    if let (Some(followers), Some(following)) = (current_followers, current_following) {
        let current_data = AccountData {
            // timestamp: Local::now(),
            followers,
            following,
        };
        println!("Current Data: {:?}", current_data);

        // 3. 讀取歷史數據並比較
        let history_file = format!("{}_history.json", username);
        let mut history: Vec<AccountData> = Vec::new();
        if let Ok(data) = fs::read_to_string(&history_file) {
            if let Ok(parsed_history) = serde_json::from_str(&data) {
                history = parsed_history;
            }
        }

        if let Some(last_data) = history.last() {
            let follower_diff = followers as i64 - last_data.followers as i64;
            let following_diff = following as i64 - last_data.following as i64;

            println!("--- Change ---");
            println!(
                "Followers: {}",
                if follower_diff >= 0 {
                    format!("+{}", follower_diff)
                } else {
                    format!("{}", follower_diff)
                }
            );
            println!(
                "Following: {}",
                if following_diff >= 0 {
                    format!("+{}", following_diff)
                } else {
                    format!("{}", following_diff)
                }
            );
        } else {
            println!("No previous data found. This is the first record.");
        }

        // 4. 儲存新數據
        history.push(current_data);
        fs::write(&history_file, serde_json::to_string_pretty(&history)?)?;
    } else {
        println!("Could not extract follower/following data. Instagram HTML structure might have changed or requires login.");
    }

    Ok(())
}
