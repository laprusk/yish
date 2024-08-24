use serde::{Serialize, Deserialize};
use tauri::Manager;
use chrono::Local;
use crate::convert_audio::convert_audio;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenConfig {
    pub count_problems: u32,
    pub gen_type: u32,
    pub speed_scale: f64,
    pub output_path: String,
}

#[tauri::command]
pub fn get_default_path() -> String {
    println!("Getting default path...");
    let mut audio_dir = dirs::audio_dir().unwrap();
    audio_dir.push("yish");

    audio_dir.to_str().unwrap().to_string()
}

#[tauri::command]
pub async fn generate_audio(
    yomiage_config: yomiage::Config,
    gen_config: GenConfig,
    app_handle: tauri::AppHandle,
) -> Result<(), tauri::Error> {
    println!("Generating audio...");
    tauri::async_runtime::spawn(async move {
        let subtractions = if gen_config.gen_type == 0 {
            0
        } else {
            yomiage_config.subtractions
        };
        
        let dir_name = get_dir_name(yomiage_config);

        for i in 0..gen_config.count_problems {
            // emit
            app_handle.emit_all(
                "progress",
                format!("生成中... {}/{}", i, gen_config.count_problems),
            )?;
            
            // 生成タイプが2のとき、加算問題と加減算問題を交互に生成
            let problem = yomiage::Problem::new(
                yomiage::Config {
                    subtractions: if gen_config.gen_type == 2 && i % 2 == 0 {
                        0
                    } else {
                        subtractions
                    },
                    ..yomiage_config
                }
            );
            let problem = match problem {
                Ok(p) => p,
                Err(e) => {
                    return Err(tauri::Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    )));
                }
                
            };
            
            let file_name = get_file_name(problem.config.subtractions, i + 1);
            convert_audio(
                &app_handle, problem, gen_config.clone(), &dir_name, &file_name
            ).expect("failed to convert audio");
        }

        // emit
        app_handle.emit_all("progress", "生成完了")?;
        app_handle.emit_all("finish", "生成完了")?;

        Ok(())
    }).await??;

    println!("Audio generated!");
    Ok(())
}

fn get_timestamp() -> String {
    Local::now().format("%Y%m%d%H%M%S").to_string()
}

fn get_dir_name(yomiage_config: yomiage::Config) -> String {
    format!("{}-{}-{}-{}",
        yomiage_config.min_digit,
        yomiage_config.max_digit,
        yomiage_config.length,
        get_timestamp()
    )
}

fn get_file_name(subtractions: u32, i: u32) -> String {
    let problem_type = if subtractions == 0 {
        String::from("加算")
    } else {
        String::from("加減算")
    };

    format!("{}-{}.wav", i, problem_type)
}