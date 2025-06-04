use std::path::PathBuf;

use clap::Args;

#[derive(Args, Default)]
pub struct DownloadOptions {
    #[arg(help = "The model detail page URL.")]
    pub url: String,
    #[arg(
        short = 'o',
        long = "output",
        help = "The directory stores the download files."
    )]
    pub output_path: Option<PathBuf>,
    #[arg(
        long = "fix-missing",
        short = 'f',
        help = "Fix missing directories.",
        default_value = "false"
    )]
    pub fix_missing_dirs: bool,
}

pub async fn process_download_options(options: &DownloadOptions) {
    let target_url = reqwest::Url::parse(&options.url).expect("The given url is invalid");

    if let Some(path) = options.output_path.as_ref() {
        if !path.exists() && options.fix_missing_dirs {
            std::fs::create_dir_all(path).expect("Failed to create output directory");
        }
    }

    let target_platform = crate::downloader::detect_platform(&target_url);

    match target_platform {
        Some(crate::downloader::Platform::Civitai) => {
            println!("Downloading from Civitai...");
            if !crate::configuration::check_civitai_key_exists().await {
                println!("Civitai access key is not set. Please set it first.");
                return;
            }
            let (model_id, model_version_id) =
                match crate::civitai::try_parse_civitai_model_url(&target_url) {
                    Ok(result) => result,
                    Err(error) => {
                        panic!("{}", error);
                    }
                };
            let civitai_client = crate::downloader::make_client()
                .await
                .expect("Failed to initialize client");
            crate::civitai::download_from_civitai(
                &civitai_client,
                model_id.parse::<u64>().expect("Failed to parse model id"),
                model_version_id
                    .map(|s| s.parse::<u64>().expect("Failed to parse model version id")),
                options.output_path.as_ref(),
            )
            .await
            .expect("Failed to download model file(s)");
            println!("Download completed.");
        }
        Some(crate::downloader::Platform::HuggingFace) => {
            if !crate::configuration::check_huggingface_key_exists().await {
                println!("HuggingFace API key is not set. Please set it first.");
                return;
            }
            println!("Downloading from HuggingFace is not supported yet.");
        }
        _ => {
            println!("Unsupported platform.");
        }
    }
}
