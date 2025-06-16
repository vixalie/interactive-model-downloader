# Interactive Model Downloader (IMD)

This is a command-line tool for downloading models from Hugging Face and Civitai. This tool can check the content contained within a given model based on the provided URL, interactively ask for the content required for download, and also retain the basic information of the model.

## Features

- Golbal comfiguration.
- Support Hugging Face and Civitai.
- Download through proxy.
- Interactive setup.
- Original API.

## Build from source

IMD is written in Rust, you need install Rust first.

### Install by Cargo

Clone the repository and navigate into it. Run `cargo install` command.

```bash
cargo install --path .
```

The tool `imd` will be installed to cargo bin directory, you may run it directly.

### Build executable

Clone this repository and run `cargo build --release` command. The executable will be generated in `target/release/imd`, you need to copy it to a directory in your `$PATH` variable.

## Usage

You can visit detail usage of "imd" tool by `imd --help` command.

### Setup api keys

Before you can download models from huggingface or civitai, you need to setup api keys. You can use `imd config set --help` command to visit which api keys you can set, and also other configurations.

### Setup proxy

Be default, imd tool will use no proxy. If you need to use proxy, you can set it by `imd config set proxy` command. For example, you can set proxy by `imd config set proxy socks5://127.0.0.1:1080`.

After setted proxy server, you need to run `imd config set enable-proxy true` to enable imd tool to use proxy.

### Download models

Download models is performed by `imd download` command. It deesn't need to specify platform, imd tool will automatically detect them.

`imd download` command take a model page url as an argument. For example, downlaoding Flux model from Civitai:

```bash
imd download 'https://civitai.com/models/618692/flux?modelVersionId=691639'
```

If the model has multiple versions, imd tool will show a list of version and ask you to select one. And also if there are multiple files in selected version, imd tool will ask you to select one or more. Whene you finished selection, imd tool will start downloading.

> Download from huggingface is not implemented yet.

### Renew model information

(Not implemented yet)

### List models

(Not implemented yet)

## License

Interactive Model Downloader (IMD) follows the Apache license 2.0. See the [LICENSE](LICENSE) file for more information.
