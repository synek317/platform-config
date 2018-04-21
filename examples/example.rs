#[macro_use]
extern crate platform_config;
use platform_config::*;

#[derive(PlatformConfig, Debug)]
struct MyAppConfig {
    // required in cmd line
    #[platformconfig(short = "s", long = "source")]
    pub source: String,

    // required in at least one of config sources, optional in cmd line
    #[platformconfig(optional_cmd, short = "d", long = "debug")]
    pub debug: bool,

    // required in at least one of config sources; cannot be passed in cmd line
    #[platformconfig(no_cmd)]
    pub db_host: String,

    // optional
    #[platformconfig(short = "t", long = "timeout")]
    pub timeout: Option<usize>,

    // optional, cannot be passed in cmd line
    #[platformconfig(no_cmd)]
    pub port: Option<u32>,

    // optional positional arg in cmd line (can be also defined in any other config source)
    pub pattern: Option<String>
}

fn main() {
    let config: MyAppConfig = PlatformConfigBuilder::new()
        .with_file("config/default.toml")
        .build();

    println!("config: {:?}", config);
}
