# s3_search

![Test](https://github.com/shoot16625/s3_search/workflows/Test/badge.svg?branch=main)
![](https://img.shields.io/crates/v/s3_search)
![](https://img.shields.io/github/v/release/shoot16625/s3_search?sort=semver)

This is a CLI that allows you to interactively search AWS S3 object path and get AWS Management Console URI.

![Image](/image/t-rec.gif)

## Installation

### Cargo Install

With Rust's package manager cargo, you can install via:

```sh
cargo install s3_search
```

### Homebrew

macOS or Linux

```sh
brew tap shoot16625/tap
brew install s3_search
```

## Usage

```sh
# search s3 object path (default region: AWS_DEFAULT_REGION)
s3s

# search s3 object path (specify region)
s3s --region us-west-2

# help
s3s --help
```

### AWS credentials

s3_search needs aws credentials, so you need to set credentials.
You can use Environment value or `"~/.aws/credentials"`.

| environment        | default value  |
| ------------------ | -------------- |
| profile            | default        |
| AWS_DEFAULT_REGION | ap-northeast-1 |

## Develop

```sh
cargo run
```
