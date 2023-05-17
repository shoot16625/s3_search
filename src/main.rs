use aws_sdk_s3::{Client, Error};
use aws_types::region::Region;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use envconfig::Envconfig;
use std::process;
use std::{thread, time};

#[derive(Envconfig)]
struct Config {
    /// AWS Region
    #[envconfig(from = "AWS_DEFAULT_REGION", default = "ap-northeast-1")]
    region: String,
}

#[derive(Parser)]
struct Args {
    /// AWS Region
    #[arg(short, long, default_value = "")]
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
async fn select_object(client: &Client, bucket: &str, prefix: &str) -> (String, bool) {
    let objects = list_objects(&client, bucket, prefix).await;
    if objects.is_empty() {
        let mut s = prefix.to_string();
        if prefix.ends_with("/") {
            // フォルダ内にファイルが一つもない場合は、フォルダ名を返す
            s.pop();
            return (s, true);
        }
        return (s, false);
    }

    let prefixes: Vec<&str> = objects.iter().map(|p| p.as_str()).collect();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&prefixes)
        .default(0)
        .interact();

    let selected_prefix = prefixes[selection.unwrap()];
    return (selected_prefix.to_string(), false);
}

/// AWS Management Console で表示するための URI を作成
async fn make_uri(bucket: &str, prefix: &str, region: &str, is_dir: bool) -> String {
    if prefix.ends_with("/") || prefix.is_empty() {
        return String::new();
    }

    let uri: String;
    if is_dir {
        uri = format!(
            // オブジェクト一覧に飛ばす
            "https://s3.console.aws.amazon.com/s3/buckets/{}?prefix={}/&region={}",
            bucket, prefix, region
        );
    } else {
        uri = format!(
            "https://s3.console.aws.amazon.com/s3/object/{}?prefix={}&region={}",
            bucket, prefix, region
        );
    }

    return uri;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 環境変数
    let config = match Config::init_from_env() {
        Ok(val) => val,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    // コマンドライン引数（優先）
    let args = Args::parse();
    let mut region = &args.region;
    if region.is_empty() {
        region = &config.region;
    }

    // バケット一覧から対象のバケットを選択
    let client = prepare_client(region).await;
    let bucket = select_bucket(&client).await;

    let mut prefix = String::new();
    let mut is_dir: bool;
    let mut uri: String;
    loop {
        // 1階層下のオブジェクト一覧を表示・選択
        let current_input = prefix.as_str();
        (prefix, is_dir) = select_object(&client, bucket.as_str(), current_input).await;
        uri = make_uri(bucket.as_str(), &prefix, region, is_dir).await;
        if !uri.is_empty() {
            break;
        }
    }

    // URIを表示
    println!("URI: {}", uri);

    Ok(())
}
