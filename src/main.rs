use aws_sdk_s3::{Client, Error};
use aws_types::region::Region;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

async fn prepare_client() -> Client {
    let region_name = "ap-northeast-1";
    let config = aws_config::from_env()
        .region(Region::new(region_name))
        .load()
        .await;
    return Client::new(&config);
}

/// バケット名一覧を取得
async fn list_buckets(client: &Client) -> Vec<String> {
    let resp = client.list_buckets().send().await.unwrap();
    let objects = resp.buckets().unwrap();
    let mut buckets: Vec<String> = Vec::with_capacity(objects.len());
    for object in objects {
        let bucket = object.name().unwrap();
        buckets.push(bucket.to_string());
    }
    return buckets;
}

/// バケット内のオブジェクト一覧を取得
/// 最大1,000件までしか取得できない
async fn list_objects(client: &Client, bucket: &str, prefix: &str) -> Vec<String> {
    let resp = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .delimiter("/") // 1階層のみ対象にする
        .send()
        .await
        .unwrap();

    let common_prefixes = resp.common_prefixes();
    if common_prefixes.is_none() {
        // ファイル一覧を取得
        let mut prefixes = Vec::with_capacity(resp.contents().unwrap().len());
        for object in resp.contents().unwrap() {
            let prefix = object.key().unwrap();
            prefixes.push(prefix.to_string());
        }
        return prefixes;
    } else {
        // ディレクトリ一覧を取得
        let mut prefixes = Vec::with_capacity(common_prefixes.unwrap().len());
        for object in common_prefixes.unwrap() {
            let prefix = object.prefix().unwrap();
            prefixes.push(prefix.to_string());
        }
        return prefixes;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = prepare_client().await;

    // バケット名一覧を取得
    let buckets = list_buckets(&client).await;
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&buckets)
        .default(0)
        .interact();
    let bucket = buckets[selection.unwrap()].as_str();

    let mut prefix = String::new();
    loop {
        let current_input = prefix.as_str();
        let objects = list_objects(&client, bucket, current_input).await;
        let prefixes: Vec<&str> = objects.iter().map(|s| s.as_str()).collect();

        // 候補リストがある場合は、選択可能なUI要素に表示する
        if !prefixes.is_empty() {
            // ユーザーに選択させるためのインタラクティブな選択肢を作成
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .items(&prefixes)
                .default(0)
                .interact();

            // ユーザーが候補リストから選択した場合は、その選択内容を現在の入力欄に自動的に挿入する
            let selected_prefix = prefixes[selection.unwrap()];
            if prefixes.contains(&selected_prefix) {
                prefix = format!(
                    "{}{}",
                    current_input,
                    &selected_prefix[current_input.len()..]
                );
            }
        }
    }

    Ok(())
}
