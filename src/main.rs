use aws_sdk_s3::{Client, Error};
use aws_types::region::Region;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

#[derive(Parser)]
struct Args {
    /// AWS Region
    #[arg(short, long, default_value = "ap-northeast-1")] // WISH:環境変数も考慮できるとよい
    region: String,

    /// デバッグモード
    #[arg(short, long)]
    debug: bool,
}

/// AWSクライアントを準備
async fn prepare_client(region: &str) -> Client {
    let config = aws_config::from_env()
        .region(Region::new(region.to_string()))
        .load()
        .await;
    return Client::new(&config);
}

/// バケット名一覧を取得
async fn list_buckets(client: &Client) -> Vec<String> {
    let resp = client.list_buckets().send().await;
    match resp {
        Ok(output) => {
            let objects = output.buckets().unwrap();
            let mut buckets: Vec<String> = Vec::with_capacity(objects.len());
            for object in objects {
                let bucket = object.name().unwrap();
                buckets.push(bucket.to_string());
            }
            return buckets;
        }
        Err(error) => {
            eprintln!("エラー: {}", error);
            return Vec::new();
        }
    }
}

/// バケット一覧の取得と選択
async fn select_bucket(client: &Client) -> String {
    let buckets = list_buckets(&client).await;
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&buckets)
        .default(0)
        .interact();
    let bucket = buckets[selection.unwrap()].to_string();
    return bucket;
}

/// バケット内のオブジェクト一覧を取得
/// 最大1,000件までしか取得できない（WISH:クロールできるとよい）
async fn list_objects(client: &Client, bucket: &str, prefix: &str) -> Vec<String> {
    let resp = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .delimiter("/") // 1階層のみ対象にする
        .send()
        .await;

    match resp {
        Ok(output) => {
            let mut prefixes: Vec<String> = Vec::with_capacity(1000);
            let common_prefixes = output.common_prefixes();
            let contents = output.contents();

            if !common_prefixes.is_none() {
                // ディレクトリ一覧を取得
                for object in common_prefixes.unwrap() {
                    let prefix = object.prefix().unwrap();
                    prefixes.push(prefix.to_string());
                }
            }

            if !contents.is_none() {
                // ファイル一覧を取得
                for object in contents.unwrap() {
                    let prefix = object.key().unwrap();
                    if prefix.ends_with("/") {
                        continue;
                    }
                    prefixes.push(prefix.to_string());
                }
            }

            return prefixes;
        }
        Err(error) => {
            eprintln!("エラー: {}", error);
            return Vec::new();
        }
    }
}

/// オブジェクトが一覧の取得と選択
async fn select_object(client: &Client, bucket: &str, prefix: &str) -> String {
    let objects = list_objects(&client, bucket, prefix).await;

    let prefixes: Vec<&str> = objects.iter().map(|p| p.as_str()).collect();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&prefixes)
        .default(0)
        .interact();

    let selected_prefix = prefixes[selection.unwrap()];
    return selected_prefix.to_string();
}

/// AWS Management Console で表示するための URI を作成
async fn make_uri(bucket: &str, prefix: &str, region: &str) -> String {
    if prefix.ends_with("/") {
        return String::new();
    }

    let uri = format!(
        "https://s3.console.aws.amazon.com/s3/object/{}?prefix={}&region={}",
        bucket, prefix, region
    );
    return uri;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let region = &args.region;

    // バケット一覧から対象のバケットを選択
    let client = prepare_client(region).await;
    let bucket = select_bucket(&client).await;

    let mut prefix = String::new();
    let mut uri: String;
    loop {
        // 1階層下のオブジェクト一覧を表示・選択
        let current_input = prefix.as_str();
        prefix = select_object(&client, bucket.as_str(), current_input).await;
        uri = make_uri(bucket.as_str(), &prefix, region).await;
        if !uri.is_empty() {
            break;
        }
    }

    // URIを表示
    println!("URI: {}", uri);

    Ok(())
}
