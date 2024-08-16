use std::sync::Arc;
use vvcapi::VoicevoxCore;
use serde::{Serialize, Deserialize};
use tauri::Manager;
use crate::convert_audio::convert_audio;

#[derive(Serialize, Deserialize, Debug)]
pub struct GenConfig {
    count_problems: u32,
    gen_type: u32,
    speed_scale: f64,
    output_path: String,
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
    core: tauri::State<'_, Arc<VoicevoxCore>>,
    app_handle: tauri::AppHandle,
) -> Result<(), tauri::Error> {
    println!("Generating audio...");
    let core = Arc::clone(&core);
    tauri::async_runtime::spawn(async move {
        let subtractions = if gen_config.gen_type == 0 {
            0
        } else {
            yomiage_config.subtractions
        };

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
            convert_audio(&core, problem, gen_config.speed_scale, &gen_config.output_path)
                .expect("failed to convert audio");
        }

        // emit
        app_handle.emit_all("progress", "生成完了")?;
        app_handle.emit_all("finish", "生成完了")?;

        Ok(())
    }).await??;

    println!("Audio generated!");
    Ok(())
}
